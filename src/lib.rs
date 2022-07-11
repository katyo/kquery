#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "doc-cfg", feature(doc_cfg))]

mod filemgr;
mod kbuild;
mod makefile;
mod metadata;
mod source;

//pub(crate) use source::Source;
pub(crate) use makefile::{MakeFile, MakeStmt};
pub(crate) use std::path::{Path, PathBuf};

pub use anyhow::{Error, Result};
pub use filemgr::{File, FileMgr};
pub use metadata::{CompatStrData, ConfigOptData, MetaData, ModuleData, ParamData, SourceData};
