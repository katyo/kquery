use crate::{FileMgr, MakeFile, MakeStmt, MetaData, Path, PathBuf, Result, SourceData};

use std::{
    collections::{BTreeSet as Set, VecDeque},
    sync::{
        atomic::{AtomicIsize, Ordering},
        Arc,
    },
};
use tokio::{
    spawn,
    sync::{
        mpsc::{channel, Sender},
        RwLock,
    },
};

#[derive(Debug)]
struct ConditionsData {
    condition: String,
    conditions: Conditions,
}

#[derive(Debug, Clone, Default)]
pub struct Conditions {
    inner: Arc<Option<ConditionsData>>,
}

impl<Rhs: AsRef<str>> core::ops::AddAssign<Rhs> for Conditions {
    fn add_assign(&mut self, new_condition: Rhs) {
        *self = self.clone() + new_condition;
    }
}

impl<Rhs: AsRef<str>> core::ops::Add<Rhs> for Conditions {
    type Output = Self;

    fn add(self, new_condition: Rhs) -> Self {
        let new_condition = new_condition.as_ref();
        Self {
            inner: Arc::new(Some(ConditionsData {
                condition: new_condition.into(),
                conditions: self,
            })),
        }
    }
}

impl<'a> From<&'a Conditions> for Set<String> {
    fn from(mut conditions: &Conditions) -> Self {
        let mut set = Set::default();
        while let Some(data) = &*conditions.inner {
            set.insert(data.condition.clone());
            conditions = &data.conditions;
        }
        set
    }
}

impl MetaData {
    /// Create metadata by indexing kbuild files and sources
    pub async fn from_kbuild(filemgr: &FileMgr) -> Result<Self> {
        let state = State::new(filemgr.clone());

        state.process().await?;

        let mut result = state.result()?;

        result.sync_with_sources();

        Ok(result)
    }
}

#[derive(Debug)]
struct StateData {
    /** File manager instance */
    filemgr: FileMgr,
    /** Processed kbuild files */
    donekbuild: RwLock<Set<PathBuf>>,
    /** Result metadata */
    metadata: RwLock<MetaData>,
}

#[derive(Debug)]
enum StateOp {
    Add(State),
    Done(Result<()>),
}

#[derive(Debug, Clone, educe::Educe)]
#[educe(Deref, DerefMut)]
struct State {
    /** Shared Data */
    #[educe(Deref, DerefMut)]
    shared: Arc<StateData>,
    /** Current directory */
    path: Arc<PathBuf>,
    /** Current conditions set */
    conditions: Conditions,
}

impl State {
    fn new(filemgr: FileMgr) -> Self {
        Self {
            shared: Arc::new(StateData {
                filemgr,
                donekbuild: RwLock::new(Set::default()),
                metadata: RwLock::new(MetaData::default()),
            }),
            path: Arc::new(PathBuf::default()),
            conditions: Conditions::default(),
        }
    }

    fn add_subdir(&mut self, name: impl AsRef<Path>) {
        self.path = Arc::new(self.path.join(name));
    }

    fn with_subdir(&self, name: impl AsRef<Path>) -> Self {
        let mut state = self.clone();
        state.add_subdir(name);
        state
    }

    fn add_condition(&mut self, condition: impl AsRef<str>) {
        self.conditions += condition;
    }

    async fn add_source(&self, path: impl AsRef<Path>, data: impl Into<SourceData>) {
        let mut data = data.into();
        data.add_config_opts(&self.conditions);

        self.metadata
            .write()
            .await
            .sources
            .insert(path.as_ref().into(), data);
    }

    async fn add_object(&self, name: impl AsRef<Path>) -> Result<()> {
        let path = self.path.join(name);
        for extension in ["c", "S"] {
            let source_path = path.with_extension(extension);
            if self.filemgr.file_exists(&source_path).await? {
                if extension == "c" {
                    match SourceData::from_source(&self.filemgr, &source_path).await {
                        Ok(source_data) => self.add_source(&source_path, source_data).await,
                        Err(error) => {
                            log::warn!("Unable to find compats for: {:?} due to: {}", path, error);
                        }
                    }
                }

                return Ok(());
            }
        }
        log::warn!("Unable to find source for: {:?}", path);
        Ok(())
    }

    fn result(self) -> Result<MetaData> {
        Ok(Arc::try_unwrap(self.shared)
            .or_else(|_| anyhow::bail!("Unable to unwrap data"))?
            .metadata
            .into_inner())
    }

    async fn process(&self) -> Result<()> {
        let cpus = std::thread::available_parallelism()?.get();
        let (tx, mut rx) = channel(cpus);
        let tasks = AtomicIsize::default();

        tx.send(StateOp::Add(self.clone())).await?;

        while let Some(op) = rx.recv().await {
            match op {
                StateOp::Done(result) => {
                    result?;
                    let count = tasks.fetch_sub(1, Ordering::SeqCst) - 1;
                    //log::trace!("done task: {}", count);
                    if count == 0 {
                        break;
                    }
                }
                StateOp::Add(state) => {
                    let tx = tx.clone();

                    tasks.fetch_add(1, Ordering::SeqCst);
                    //let count = tasks.fetch_add(1, Ordering::SeqCst) + 1;
                    //log::trace!("add task: {}", count);

                    spawn(async move {
                        if let Err(err) = tx
                            .send(StateOp::Done(state.process_dir(tx.clone()).await))
                            .await
                        {
                            eprintln!("Unable to finalize task due to: {}", err);
                        }
                    });
                }
            }
        }

        Ok(())
    }

    async fn process_dir(&self, tx: Sender<StateOp>) -> Result<()> {
        let files = ["Kbuild", "Makefile"];

        for name in &files {
            let path = self.path.join(name);

            if self.donekbuild.read().await.contains(&path) {
                return Ok(());
            }

            self.donekbuild.write().await.insert(path.clone());

            if self.filemgr.file_exists(&path).await? {
                self.process_makefile(&path, &tx).await?;
            }
        }

        log::error!("Missing files: {:?} at {:?}", files, self.path);

        Ok(())
    }

    async fn process_makefile(&self, path: impl AsRef<Path>, tx: &Sender<StateOp>) -> Result<()> {
        let mut makefile = MakeFile::parse(&self.filemgr, path).await?;

        let mut stack = VecDeque::default();
        stack.push_back(self.clone());

        while let Some(stmt) = makefile.next_stmt().await? {
            log::trace!("Make statement: {:?}", stmt);
            match stmt {
                MakeStmt::Var {
                    conditions,
                    elements,
                    ..
                } => {
                    let mut state = stack.back().unwrap().clone();
                    for condition in conditions {
                        state.add_condition(condition);
                    }
                    for element in &elements {
                        let name = Path::new(element);
                        if let Some(extension) = name.extension() {
                            if extension == "o" {
                                state.add_object(name).await?;
                            }
                        } else {
                            let state = state.clone();
                            tx.send(StateOp::Add(state.with_subdir(name))).await?;
                        }
                    }
                }
                MakeStmt::If { conditions } => {
                    let mut state = stack.back().unwrap().clone();
                    for condition in conditions {
                        state.add_condition(condition);
                    }
                    stack.push_back(state);
                }
                MakeStmt::EndIf => {
                    if stack.len() > 1 {
                        stack.pop_back();
                    }
                }
                MakeStmt::ElseIf { conditions } => {
                    if stack.len() > 1 {
                        stack.pop_back();
                    }
                    let mut state = stack.back().unwrap().clone();
                    for condition in conditions {
                        state.add_condition(condition);
                    }
                    stack.push_back(state);
                }
            }
        }

        Ok(())
    }
}
