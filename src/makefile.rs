use crate::{File, FileMgr, Path, Result};
use std::collections::{HashMap as Map, VecDeque as Deque};

struct Orphan {
    elements: Vec<String>,
    conditions: Vec<String>,
}

pub struct MakeFile {
    lines: tokio::io::Lines<tokio::io::BufReader<File>>,
    // module_name => condition[]
    modules: Map<String, Vec<String>>,
    // module_name => (elements, conditions)[]
    orphans: Map<String, Deque<Orphan>>,
    // unprocessed module
    module: Option<String>,
}

impl MakeFile {
    pub async fn parse(filemgr: &FileMgr, path: impl AsRef<Path>) -> Result<Self> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        log::debug!("parse kbuild file: {:?}", path.as_ref());

        let file = filemgr.open(&path).await?;
        let lines = BufReader::new(file).lines();

        Ok(Self {
            lines,
            modules: Map::default(),
            orphans: Map::default(),
            module: None,
        })
    }

    pub async fn next_stmt(&mut self) -> Result<Option<MakeStmt>> {
        if let Some(prefix) = &self.module {
            if let (Some(module_conditions), Some(queue)) =
                (self.modules.get(prefix), self.orphans.get_mut(prefix))
            {
                if let Some(Orphan {
                    elements,
                    mut conditions,
                }) = queue.pop_front()
                {
                    conditions.extend(module_conditions.clone());
                    let prefix = prefix.clone();
                    return Ok(Some(MakeStmt::Var {
                        prefix,
                        elements,
                        conditions,
                    }));
                }
                self.orphans.remove(prefix);
            }
            self.module = None;
        }

        let mut full_line: Option<String> = None;

        while let Some(line) = self.lines.next_line().await? {
            if let Some((line, "")) = line.rsplit_once('\\') {
                if let Some(full_line) = &mut full_line {
                    full_line.push_str(line);
                } else {
                    full_line = Some(line.into());
                }
                continue;
            }

            let line = if let Some(mut full_line) = full_line.take() {
                full_line.push_str(&line);
                full_line
            } else {
                line
            };

            match MakeStmt::parse(&line) {
                Ok(Some(mut stmt)) => {
                    if let MakeStmt::Var {
                        prefix,
                        elements,
                        conditions,
                    } = &mut stmt
                    {
                        if ["obj", "lib", "subdir", "core", "drivers"]
                            .into_iter()
                            .any(|entry| entry == prefix)
                        {
                            for element in elements {
                                if let Some((module, "")) = element.rsplit_once(".o") {
                                    if self.orphans.contains_key(module) {
                                        self.module = Some(module.into());
                                    }
                                    self.modules.insert(module.into(), conditions.clone());
                                }
                            }
                        } else if let Some(module_conditions) = self.modules.get(prefix) {
                            conditions.extend(module_conditions.clone());
                        } else {
                            if let Some(queue) = self.orphans.get_mut(prefix) {
                                queue.push_back(Orphan {
                                    elements: elements.clone(),
                                    conditions: conditions.clone(),
                                });
                            } else {
                                let mut queue = Deque::default();
                                queue.push_back(Orphan {
                                    elements: elements.clone(),
                                    conditions: conditions.clone(),
                                });
                                self.orphans.insert(prefix.clone(), queue);
                            }
                            continue;
                        }
                    }

                    return Ok(Some(stmt));
                }
                Ok(None) => {
                    continue;
                }
                Err(err) => {
                    log::trace!("MakeStmt::parse fail: {}", err);
                    continue;
                }
            }
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub enum MakeStmt {
    Var {
        prefix: String,
        conditions: Vec<String>,
        elements: Vec<String>,
    },
    If {
        conditions: Vec<String>,
    },
    ElseIf {
        conditions: Vec<String>,
    },
    EndIf,
}

impl MakeStmt {
    fn parse_conditions(st: &str) -> Vec<String> {
        st.split("$(CONFIG_")
            .skip(1)
            .filter_map(|cond| cond.split_once(')').map(|(var, _)| var.trim().to_string()))
            .filter(|el| el.chars().all(|c: char| c.is_alphanumeric() || c == '_'))
            .collect()
    }

    fn parse_elements<'a>(pfx: &str, st: &'a str) -> Vec<String> {
        st.split(char::is_whitespace)
            .filter_map(if pfx != "subdir" {
                |el: &'a str| {
                    if el.ends_with('/') || el.ends_with(".o") {
                        Some(el)
                    } else {
                        None
                    }
                }
            } else {
                Some
            })
            .map(|el| el.trim_end_matches('/'))
            .filter(|el| {
                !el.starts_with('-')
                    && el.chars().all(|c: char| {
                        c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/'
                    })
            })
            .map(String::from)
            .collect()
    }

    fn parse(line: &str) -> Result<Option<Self>> {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            /* skip entry lines and makefile comments */
            return Ok(None);
        }

        if line.starts_with("endif") {
            return Ok(Some(Self::EndIf));
        }

        if line.starts_with("else ") {
            let line = line.split_once(char::is_whitespace).unwrap().1;

            if line.starts_with("ifdef ") || line.starts_with("ifndef ") {
                let cond = line.split_once(char::is_whitespace).unwrap().1;
                if let Some((_, cond)) = cond.split_once("CONFIG_") {
                    let conditions = vec![cond.into()];
                    return Ok(Some(Self::ElseIf { conditions }));
                }
            } else if line.starts_with("ifeq ") || line.starts_with("ifneq ") {
                let conditions =
                    Self::parse_conditions(line.split_once(char::is_whitespace).unwrap().1);

                return Ok(Some(Self::ElseIf { conditions }));
            }
        } else if line.starts_with("ifdef ") || line.starts_with("ifndef ") {
            let cond = line.split_once(char::is_whitespace).unwrap().1;
            if let Some((_, cond)) = cond.split_once("CONFIG_") {
                let conditions = vec![cond.into()];
                return Ok(Some(Self::If { conditions }));
            }
        } else if line.starts_with("ifeq ") || line.starts_with("ifneq ") {
            let conditions =
                Self::parse_conditions(line.split_once(char::is_whitespace).unwrap().1);

            return Ok(Some(Self::If { conditions }));
        } else if let Some((pfx, key, val)) = line.split_once('=').and_then(|(var, val)| {
            split_once_if(var, "-", |_, sfx| {
                sfx.starts_with("$(") || !sfx.contains('-')
            })
            .map(|(pfx, key)| (pfx, key, val))
        }) {
            let conditions =
                Self::parse_conditions(key.trim_end_matches(|c: char| {
                    c == '+' || c == ':' || c == '?' || c.is_whitespace()
                }));

            let elements = Self::parse_elements(pfx, val.trim_start());

            if elements.is_empty() {
                return Ok(None);
            }

            return Ok(Some(Self::Var {
                prefix: pfx.into(),
                conditions,
                elements,
            }));
        }

        anyhow::bail!("{:?}", line);
    }
}

fn split_once_if<'a>(
    s: &'a str,
    p: &'a str,
    f: impl Fn(&'a str, &'a str) -> bool,
) -> Option<(&'a str, &'a str)> {
    for (idx, _) in s.match_indices(p) {
        let a = &s[..idx];
        let b = &s[idx + p.len()..];
        if f(a, b) {
            return Some((a, b));
        }
    }
    None
}
