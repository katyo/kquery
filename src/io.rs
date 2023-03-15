use crate::{filemgr, Error, MetaData, Path, PathBuf, Result};

/// Metadata coding format
#[cfg(any(feature = "json", feature = "cbor"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, educe::Educe)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[educe(Default)]
pub enum DataCoding {
    /// JSON format
    #[cfg(feature = "json")]
    #[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "json")))]
    #[educe(Default)]
    Json,

    /// JSON format (pretty printed)
    #[cfg(feature = "json")]
    #[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "json")))]
    JsonPretty,

    /// CBOR format
    #[cfg(feature = "cbor")]
    #[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "cbor")))]
    #[cfg_attr(not(feature = "json"), educe(Default))]
    Cbor,
}

impl From<&DataCoding> for DataCoding {
    fn from(r: &Self) -> Self {
        *r
    }
}

impl core::str::FromStr for DataCoding {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            #[cfg(feature = "json")]
            "j" | "json" => Self::Json,
            #[cfg(feature = "json")]
            "jp" | "json-pretty" => Self::JsonPretty,
            #[cfg(feature = "cbor")]
            "c" | "cbor" => Self::Cbor,
            _ => anyhow::bail!("Insupported data coding: {}", s),
        })
    }
}

impl AsRef<str> for DataCoding {
    fn as_ref(&self) -> &str {
        match self {
            #[cfg(feature = "json")]
            Self::Json => "json",
            #[cfg(feature = "json")]
            Self::JsonPretty => "json-pretty",
            #[cfg(feature = "cbor")]
            Self::Cbor => "cbor",
        }
    }
}

impl core::fmt::Display for DataCoding {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl DataCoding {
    pub const POSSIBLE_STRS: &'static [&'static str] = &[
        #[cfg(feature = "json")]
        "json",
        #[cfg(feature = "json")]
        "json-pretty",
        #[cfg(feature = "cbor")]
        "cbor",
    ];
}

/// Metadata compression
#[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, educe::Educe)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[educe(Default)]
pub enum DataCompress {
    /// No compress
    #[educe(Default)]
    No,

    /// LZ4 compression
    #[cfg(feature = "lz4")]
    #[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "lz4")))]
    Lz4,
}

impl From<&DataCompress> for DataCompress {
    fn from(r: &Self) -> Self {
        *r
    }
}

impl core::str::FromStr for DataCompress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "n" | "no" => Self::No,
            #[cfg(feature = "lz4")]
            "z" | "lz4" => Self::Lz4,
            _ => anyhow::bail!("Insupported data compression: {}", s),
        })
    }
}

impl AsRef<str> for DataCompress {
    fn as_ref(&self) -> &str {
        match self {
            Self::No => "no",
            #[cfg(feature = "lz4")]
            Self::Lz4 => "lz4",
        }
    }
}

impl core::fmt::Display for DataCompress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl DataCompress {
    pub const POSSIBLE_STRS: &'static [&'static str] = &[
        "no",
        #[cfg(feature = "lz4")]
        "lz4",
    ];
}

/// Metadata options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DataOptions {
    /// Data coding format
    pub coding: DataCoding,

    /// Data compression
    pub compress: DataCompress,
}

impl DataOptions {
    /// Create data options
    pub fn new(coding: impl Into<DataCoding>, compress: impl Into<DataCompress>) -> Self {
        Self {
            coding: coding.into(),
            compress: compress.into(),
        }
    }

    /// Infer options from metadata file name
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let mut path = path.as_ref();

        let mut exts = core::iter::from_fn(move || {
            path.file_stem().and_then(|stem| {
                let ext = path.extension().and_then(|ext| ext.to_str());
                path = Path::new(stem);
                ext
            })
        });

        let (coding, compress) = exts
            .next()
            .map(|ext| (ext.parse().ok(), ext.parse().unwrap_or_default()))
            .unwrap_or_else(|| (None, Default::default()));

        let coding = coding.or_else(|| exts.next().and_then(|ext| ext.parse().ok()));

        if let Some(coding) = coding {
            log::trace!("File compress: {}, coding: {}", compress, coding);

            Ok(Self { coding, compress })
        } else {
            Err(anyhow::anyhow!(
                "Unable to determine data coding by file name"
            ))
        }
    }

    /// Get metadata file name corresponding to options
    pub fn file_name(&self) -> &'static str {
        match (self.coding, self.compress) {
            #[cfg(feature = "json")]
            (DataCoding::Json | DataCoding::JsonPretty, DataCompress::No) => {
                concat!(env!("CARGO_PKG_NAME"), ".json")
            }
            #[cfg(feature = "cbor")]
            (DataCoding::Cbor, DataCompress::No) => {
                concat!(env!("CARGO_PKG_NAME"), ".cbor")
            }
            #[cfg(all(feature = "json", feature = "lz4"))]
            (DataCoding::Json | DataCoding::JsonPretty, DataCompress::Lz4) => {
                concat!(env!("CARGO_PKG_NAME"), ".json.lz4")
            }
            #[cfg(all(feature = "cbor", feature = "lz4"))]
            (DataCoding::Cbor, DataCompress::Lz4) => {
                concat!(env!("CARGO_PKG_NAME"), ".cbor.lz4")
            }
        }
    }

    /// Possible metadata file names
    pub const FILE_NAMES: &'static [&'static str] = &[
        #[cfg(feature = "json")]
        concat!(env!("CARGO_PKG_NAME"), ".json"),
        #[cfg(feature = "cbor")]
        concat!(env!("CARGO_PKG_NAME"), ".cbor"),
        #[cfg(all(feature = "json", feature = "lz4"))]
        concat!(env!("CARGO_PKG_NAME"), ".json.lz4"),
        #[cfg(all(feature = "cbor", feature = "lz4"))]
        concat!(env!("CARGO_PKG_NAME"), ".cbor.lz4"),
    ];
}

impl MetaData {
    /// Load metadata from raw
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub fn from_raw(data: &[u8], opts: &DataOptions) -> Result<Self> {
        #[cfg(feature = "lz4_flex")]
        let data: std::borrow::Cow<[u8]> = match opts.compress {
            DataCompress::No => data.into(),
            DataCompress::Lz4 => lz4_flex::decompress_size_prepended(data)?.into(),
        };

        let data = std::io::Cursor::new(data);

        let mut data: Self = match opts.coding {
            #[cfg(feature = "json")]
            DataCoding::Json | DataCoding::JsonPretty => serde_json::from_reader(data)?,
            #[cfg(feature = "cbor")]
            DataCoding::Cbor => ciborium::de::from_reader(data)?,
        };

        data.sync_with_sources();

        Ok(data)
    }

    /// Load metadata from reader
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn from_reader(
        mut reader: impl tokio::io::AsyncRead + Unpin,
        opts: &DataOptions,
    ) -> Result<Self> {
        use tokio::io::AsyncReadExt;

        let mut data = Vec::default();

        reader.read_to_end(&mut data).await?;

        Self::from_raw(&data, opts)
    }

    /// Load metadata from file
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn from_file(
        path: impl AsRef<Path>,
        opts: Option<&DataOptions>,
    ) -> Result<Option<Self>> {
        let path = path.as_ref();

        // keep to hold ref
        #[allow(unused_assignments)]
        let mut opts_owned = None;

        let opts = if let Some(opts) = opts {
            opts
        } else {
            opts_owned = Some(DataOptions::from_file(path)?);
            opts_owned.as_ref().unwrap()
        };

        if !filemgr::file_exists(path).await {
            return Ok(None);
        }

        let file = tokio::fs::File::open(path).await?;

        Self::from_reader(file, opts).await.map(Some)
    }

    /// Find latest metadata file
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn find_file(path: impl AsRef<Path>) -> Result<Option<PathBuf>> {
        let path = path.as_ref();

        let mut last_found = None;

        for file_name in DataOptions::FILE_NAMES {
            let file_path = path.join(file_name);

            log::trace!("Try metadata file: {}", file_path.display());

            if let Some(mtime) = filemgr::file_mtime(&file_path).await {
                log::trace!(
                    "Metadata file modified at: {:?}",
                    mtime
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                );

                if last_found
                    .as_ref()
                    .map(|(_, last_mtime)| last_mtime < &mtime)
                    .unwrap_or(true)
                {
                    last_found = Some((file_path, mtime));
                }
            }
        }

        Ok(last_found.map(|(path, _)| {
            log::debug!("Found metadata file: {}", path.display());
            path
        }))
    }

    /// Load metadata from directory
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn from_dir(
        path: impl AsRef<Path>,
        opts: Option<&DataOptions>,
    ) -> Result<Option<Self>> {
        let path = path.as_ref();

        let path = if let Some(opts) = opts {
            path.join(opts.file_name())
        } else if let Some(path) = Self::find_file(path).await? {
            path
        } else {
            return Ok(None);
        };

        Self::from_file(path, opts).await
    }

    /// Load metadata from path
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn from_path(
        path: impl AsRef<Path>,
        opts: Option<&DataOptions>,
    ) -> Result<Option<Self>> {
        let path = path.as_ref();

        log::trace!("Load metadata from path: {}", path.display());

        if filemgr::dir_exists(path).await {
            Self::from_dir(path, opts).await
        } else {
            Self::from_file(path, opts).await
        }
    }

    /// Dump metadata into raw
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub fn to_raw(&self, opts: &DataOptions) -> Result<Vec<u8>> {
        let mut data = Vec::default();

        match opts.coding {
            #[cfg(feature = "json")]
            DataCoding::Json => serde_json::to_writer(&mut data, self)?,
            #[cfg(feature = "json")]
            DataCoding::JsonPretty => serde_json::to_writer_pretty(&mut data, self)?,
            #[cfg(feature = "cbor")]
            DataCoding::Cbor => ciborium::ser::into_writer(self, &mut data)?,
        }

        #[cfg(feature = "lz4_flex")]
        let data = match opts.compress {
            DataCompress::No => data,
            DataCompress::Lz4 => lz4_flex::compress_prepend_size(&data),
        };

        Ok(data)
    }

    /// Dump metadata to writer
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn to_writer(
        &self,
        mut writer: impl tokio::io::AsyncWrite + Unpin,
        opts: &DataOptions,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let data = self.to_raw(opts)?;

        writer.write_all(&data).await?;

        Ok(())
    }

    /// Dump metadata into file
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn to_file(&self, path: impl AsRef<Path>, opts: Option<&DataOptions>) -> Result<()> {
        let path = path.as_ref();

        let mut file = tokio::fs::File::create(path).await?;

        // keep to hold ref
        #[allow(unused_assignments)]
        let mut opts_owned = None;

        let opts = if let Some(opts) = opts {
            opts
        } else {
            opts_owned = Some(DataOptions::from_file(path)?);
            opts_owned.as_ref().unwrap()
        };

        self.to_writer(&mut file, opts).await?;

        Ok(())
    }

    /// Dump metadata into directory
    ///
    /// File name will be selected according options like so:
    ///
    /// - kquery.json
    /// - kquery.cbor
    /// - kquery.json.lz4
    /// - kquery.cbor.lz4
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn to_dir(&self, path: impl AsRef<Path>, opts: &DataOptions) -> Result<()> {
        self.to_file(path.as_ref().join(opts.file_name()), Some(opts))
            .await
    }

    /// Dump metadata into path
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn to_path(&self, path: impl AsRef<Path>, opts: &DataOptions) -> Result<()> {
        let path = path.as_ref();

        if filemgr::dir_exists(path).await {
            self.to_dir(path, opts).await
        } else {
            self.to_file(path, Some(opts)).await
        }
    }
}
