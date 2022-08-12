use kquery::{DataCoding, DataCompress, Result};
use std::path::PathBuf;

/// Commmand-line arguments
#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Data path [default: current working directory]
    #[clap(short, long, value_parser)]
    data_path: Option<PathBuf>,

    /// Command to run
    #[clap(subcommand)]
    pub command: Cmd,
}

impl Args {
    pub fn data_path(&self) -> Result<PathBuf> {
        Ok(if let Some(path) = &self.data_path {
            path.clone()
        } else {
            std::env::current_dir()?
        })
    }

    pub fn source(&self) -> Result<PathBuf> {
        if let Cmd::Index {
            source: Some(source),
            ..
        } = &self.command
        {
            Ok(source.clone())
        } else {
            self.data_path()
        }
    }
}

/// Commmand-line arguments
#[derive(Debug, clap::Subcommand)]
pub enum Cmd {
    /// Create or update index
    Index {
        /// Source root directory [default: data path]
        #[clap(short, long, value_parser)]
        source: Option<PathBuf>,

        /// Data coding
        #[clap(short = 'f', long, env = "KQUERY_CODING", value_parser, default_value_t = DataCoding::default(), possible_values = DataCoding::POSSIBLE_STRS)]
        coding: DataCoding,

        /// Data compression
        #[clap(short = 'z', long, env = "KQUERY_COMPRESS", value_parser, default_value_t = DataCompress::default(), possible_values = DataCompress::POSSIBLE_STRS)]
        compress: DataCompress,
    },

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
