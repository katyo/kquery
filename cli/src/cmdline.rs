use kquery::{DataCoding, DataCompress, Result};
use std::path::PathBuf;

/// Commmand-line arguments
#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Args {
    /// Source root [default: current working directory]
    #[structopt(short, long)]
    source_root: Option<PathBuf>,

    /// Data coding
    #[structopt(short = "f", long, env = "KQUERY_CODING", default_value = "cbor", possible_values = DataCoding::POSSIBLE_STRS)]
    pub coding: DataCoding,

    /// Data compression
    #[structopt(short = "z", long, env = "KQUERY_COMPRESS", default_value = "lz4", possible_values = DataCompress::POSSIBLE_STRS)]
    pub compress: DataCompress,

    /// Command to run
    #[structopt(subcommand)]
    pub command: Cmd,
}

impl Args {
    pub fn source_root(&self) -> Result<PathBuf> {
        Ok(if let Some(path) = &self.source_root {
            path.clone()
        } else {
            std::env::current_dir()?
        })
    }
}

/// Commmand-line arguments
#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Cmd {
    /// Create or update index
    #[structopt()]
    Index,

    /// List of processed sources
    #[structopt()]
    Sources {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter paths
        #[structopt(name = "glob-pattern")]
        pattern: Option<String>,
    },

    /// List of known compatible string
    #[structopt()]
    Compats {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter strings
        #[structopt()]
        pattern: Option<String>,
    },

    /// List of known configuration options
    #[structopt()]
    Configs {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter options
        #[structopt()]
        pattern: Option<String>,
    },

    /// Query source info by compatible string
    #[structopt()]
    Compat {
        /// Compatible string
        #[structopt(name = "compat-string")]
        compat: String,
    },

    /// Query sources info by configuraton option
    #[structopt()]
    Config {
        /// Configuration option
        #[structopt(name = "CONFIG_OPTION")]
        config: String,
    },

    /// Query source info by path
    #[structopt()]
    Source {
        /// Source path
        #[structopt(name = "path/to/source.c")]
        source: PathBuf,
    },
}
