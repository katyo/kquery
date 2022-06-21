use kquery::Result;
use std::path::PathBuf;

/// Commmand-line arguments
#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Args {
    /// Source root (current working directory by default)
    #[structopt(short, long)]
    source_root: Option<PathBuf>,

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
