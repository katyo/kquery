use kquery::{DataCoding, DataCompress};
use std::path::PathBuf;

/// Commmand-line arguments
#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Data path
    #[arg(short, long, value_hint = clap::ValueHint::DirPath, default_value = CurrentDir)]
    pub data_path: PathBuf,

    /// Command to run
    #[clap(subcommand)]
    pub command: Cmd,
}

/// Commmand-line arguments
#[derive(Debug, clap::Parser)]
pub enum Cmd {
    /// Create or update index
    Index {
        /// Source root directory
        #[arg(short, long, value_hint = clap::ValueHint::DirPath, default_value = CurrentDir)]
        source: PathBuf,

        /// Data coding
        #[arg(short = 'f', long, env = "KQUERY_CODING", value_enum, default_value_t = DataCoding::default())]
        coding: DataCoding,

        /// Data compression
        #[arg(short = 'z', long, env = "KQUERY_COMPRESS", value_enum, default_value_t = DataCompress::default())]
        compress: DataCompress,
    },

    /// List of processed sources
    Sources {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter paths
        #[arg(name = "glob-pattern")]
        pattern: Option<String>,
    },

    /// List of known compatible string
    Compats {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter strings
        #[arg(value_parser)]
        pattern: Option<String>,
    },

    /// List of known configuration options
    Configs {
        #[cfg(feature = "glob")]
        /// Optional pattern to filter options
        #[arg(value_parser)]
        pattern: Option<String>,
    },

    /// Query source info by compatible string
    Compat {
        /// Compatible string
        #[arg(value_parser, name = "compat-string")]
        compat: String,
    },

    /// Query sources info by configuraton option
    Config {
        /// Configuration option
        #[arg(value_parser, name = "CONFIG_OPTION")]
        config: String,
    },

    /// Query source info by path
    Source {
        /// Source path
        #[arg(value_parser, name = "path/to/source.c")]
        source: PathBuf,
    },
}

struct CurrentDir;

impl clap::builder::IntoResettable<clap::builder::OsStr> for CurrentDir {
    fn into_resettable(self) -> clap::builder::Resettable<clap::builder::OsStr> {
        std::env::current_dir()
            .map(|path| path.into_os_string().into())
            .map(clap::builder::Resettable::Value)
            .unwrap_or(clap::builder::Resettable::Reset)
    }
}
