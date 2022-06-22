mod cmdline;

use cmdline::{Args, Cmd};
use kquery::{FileMgr, MetaData, Result, SourceData};

#[paw::main]
#[tokio::main]
async fn main(args: Args) -> Result<()> {
    #[cfg(feature = "lovely_env_logger")]
    lovely_env_logger::init_default();

    log::trace!("Cmdline Args: {:?}", args);

    let filemgr = FileMgr::new(args.source_root()?)?;

    match &args.command {
        Cmd::Index => {
            println!("Creating index for {:?}...", filemgr.base_path());

            let db = MetaData::from_kbuild(&filemgr).await?;

            db.save_cache(&filemgr).await?;

            println!("Done!");

            #[cfg(feature = "alert-orphan-sources")]
            {
                use futures_lite::StreamExt;

                let mut entries = async_walkdir::WalkDir::new(filemgr.base_path());
                while let Some(entry) = entries.next().await.transpose()? {
                    let path = entry.path();
                    let path = path.strip_prefix(filemgr.base_path())?;

                    if path
                        .extension()
                        .map(|extension| extension == "c")
                        .unwrap_or(false)
                        && !path.ends_with(".mod.c")
                    /*&& entry
                    .file_type()
                    .await
                    .map(|file_type| file_type.is_file())
                    .unwrap_or(false)*/
                    {
                        if db.source(&path).is_none() {
                            log::warn!("Orphan source: {}", path.display());
                        }
                    }
                }
            }
        }

        cmd => {
            if let Some(db) = MetaData::from_cache(&filemgr).await? {
                fn print_source_data(ident: &str, source_data: &SourceData) {
                    if !source_data.config_opts.is_empty() {
                        println!("{}Configuration options:", ident);
                        for condition in &source_data.config_opts {
                            println!("{}    {}", ident, condition);
                        }
                    }
                    if !source_data.compat_strs.is_empty() {
                        println!("{}Compatible strings:", ident);
                        for compat in &source_data.compat_strs {
                            println!("{}    {}", ident, compat);
                        }
                    }
                }

                fn print_filtered_list<P: AsRef<std::path::Path>, S: AsRef<str>>(
                    entries: impl Iterator<Item = P>,
                    pattern: Option<S>,
                ) -> Result<()> {
                    #[cfg(feature = "glob")]
                    let pattern = pattern
                        .map(|pattern| {
                            globset::Glob::new(pattern.as_ref()).map(|glob| glob.compile_matcher())
                        })
                        .transpose()?;

                    #[cfg(feature = "glob")]
                    let entries = if let Some(pattern) = &pattern {
                        either::Either::Left(entries.filter(|path| pattern.is_match(path)))
                    } else {
                        either::Either::Right(entries)
                    };

                    for entry in entries {
                        println!("{}", entry.as_ref().display());
                    }

                    Ok(())
                }

                match cmd {
                    Cmd::Index => unreachable!(),

                    Cmd::Sources {
                        #[cfg(feature = "glob")]
                        pattern,
                    } => {
                        print_filtered_list(db.sources.keys(), pattern.as_ref())?;
                    }

                    Cmd::Compats {
                        #[cfg(feature = "glob")]
                        pattern,
                    } => {
                        print_filtered_list(db.compat_strs.keys(), pattern.as_ref())?;
                    }

                    Cmd::Configs {
                        #[cfg(feature = "glob")]
                        pattern,
                    } => {
                        print_filtered_list(db.config_opts.keys(), pattern.as_ref())?;
                    }

                    Cmd::Compat { compat } => {
                        if let Some(compat_data) = db.compat_str(&compat) {
                            println!("Source: {}", compat_data.source.display());
                            if let Some(source_data) = db.source(&compat_data.source) {
                                print_source_data("", source_data);
                            }
                        } else {
                            eprintln!("Compatible string \"{}\" not found!", compat);
                        }
                    }

                    Cmd::Config { config } => {
                        if let Some(config_data) = db.config_opt(&config) {
                            if !config_data.sources.is_empty() {
                                println!("Sources:");
                                for source in &config_data.sources {
                                    println!("    {}", source.display());
                                    if let Some(source_data) = db.source(source) {
                                        print_source_data("        ", source_data);
                                    }
                                }
                            } else {
                                eprintln!(
                                    "No sources related to configuration option \"{}\" found!",
                                    config
                                );
                            }
                        } else {
                            eprintln!("Configuration option \"{}\" not found!", config);
                        }
                    }

                    Cmd::Source { source } => {
                        if let Some(source_data) = db.source(&source) {
                            println!("Source: {}", source.display());
                            print_source_data("    ", source_data);
                        } else {
                            eprintln!("Source file \"{}\" not found!", source.display());
                        }
                    }
                }
            } else {
                eprintln!("Index does not exists!");
                eprintln!("Please run `kquery index` first...");
            }
        }
    };

    Ok(())
}
