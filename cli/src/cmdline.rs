use kquery::{DataCoding, DataCompress, Result};
use std::path::PathBuf;

/// Commmand-line arguments
#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Source root [default: current working directory]
    #[clap(short, long, value_parser)]
    source_root: Option<PathBuf>,

    /// Data coding
    #[clap(short = 'f', long, env = "KQUERY_CODING", value_parser, default_value_t = DataCoding::default(), possible_values = DataCoding::POSSIBLE_STRS)]
    pub coding: DataCoding,

    /// Data compression
    #[clap(short = 'z', long, env = "KQUERY_COMPRESS", value_parser, default_value_t = DataCompress::default(), possible_values = DataCompress::POSSIBLE_STRS)]
    pub compress: DataCompress,

    /// Command to run
    #[clap(subcommand)]
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
#[derive(Debug, clap::Subcommand)]
pub enum Cmd {
    /// Create or update index
    Index,

    /// List of processed sources
    Sources {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter paths
        #[clap(name = "glob-pattern")]
        pattern: Option<String>,
    },

    /// List of known compatible string
    Compats {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter strings
        #[clap(value_parser)]
        pattern: Option<String>,
    },

    /// List of known configuration options
    Configs {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter options
        #[clap(value_parser)]
        pattern: Option<String>,
    },

    /// Query source info by compatible string
    Compat {
        /// Compatible string
        #[clap(value_parser, name = "compat-string")]
        compat: String,
    },

    /// Query sources info by configuraton option
    Config {
        /// Configuration option
        #[clap(value_parser, name = "CONFIG_OPTION")]
        config: String,
    },

    /// Query source info by path
    Source {
        /// Source path
        #[clap(value_parser, name = "path/to/source.c")]
        source: PathBuf,
    },
}
