use crate::{FileMgr, Path, Result};

use std::collections::BTreeSet as Set;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct Compats {
    compats: Set<String>,
}

impl From<Compats> for Set<String> {
    fn from(compats: Compats) -> Self {
        compats.compats
    }
}

impl Compats {
    pub async fn from_source(filemgr: &FileMgr, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = filemgr.open(&path).await?;
        let mut lines = BufReader::new(file).lines();

        #[derive(Clone, Copy)]
        enum Find {
            Compat,
            Assign,
            String,
        }

        let mut compats = Set::default();
        let mut state = Find::Compat;

        while let Some(line) = lines.next_line().await? {
            let mut slice: &str = line.trim_start();
            loop {
                match state {
                    Find::Compat => {
                        if let Some((_, tail)) = slice.split_once(".compatible") {
                            state = Find::Assign;
                            slice = tail.trim_start();
                            log::trace!("compat: |{}|", &slice);
                            continue;
                        }
                        break;
                    }
                    Find::Assign => {
                        if let Some(("", tail)) = slice.split_once('=') {
                            state = Find::String;
                            slice = tail.trim_start();
                            log::trace!("assign: |{}|", &slice);
                            continue;
                        }
                        break;
                    }
                    Find::String => {
                        state = Find::Compat;
                        if let Some(("", tail)) = slice.split_once('"') {
                            slice = tail;
                            if let Some((compat, tail)) = slice.split_once('"') {
                                if !compat.is_empty() {
                                    compats.insert(compat.into());
                                }
                                slice = tail;
                                log::trace!("string: \"{}\" |{}|", compat, &slice);
                                continue;
                            }
                            log::trace!("string: |{}|", &slice);
                            continue;
                        }
                        log::warn!("Parse error: \" expected ({:?})", path);
                        break;
                    }
                }
            }
        }

        Ok(Self { compats })
    }
}
