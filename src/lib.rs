#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "doc-cfg", feature(doc_cfg))]

mod filemgr;
mod kbuild;
mod makefile;
mod metadata;
mod source;

#[cfg(any(feature = "json", feature = "cbor"))]
mod io;

pub(crate) use makefile::{MakeFile, MakeStmt};
pub(crate) use std::path::{Path, PathBuf};

pub use anyhow::{Error, Result};
pub use filemgr::{File, FileMgr};
pub use metadata::{CompatStrData, ConfigOptData, MetaData, ModuleData, ParamData, SourceData};

#[cfg(any(feature = "json", feature = "cbor"))]
pub use io::{DataCoding, DataCompress, DataOptions};
