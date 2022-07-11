use crate::{Error, MetaData, Path, Result};

/// Metadata coding format
#[cfg(any(feature = "json", feature = "cbor"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, educe::Educe)]
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

impl core::str::FromStr for DataCompress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "n" | "no" => Self::No,
            #[cfg(feature = "lz4")]
            "c" | "cbor" => Self::Lz4,
            _ => anyhow::bail!("Insupported data compression: {}", s),
        })
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
    pub async fn from_file(path: impl AsRef<Path>, opts: &DataOptions) -> Result<Option<Self>> {
        let path = path.as_ref();

        if !crate::filemgr::file_exists(path).await {
            return Ok(None);
        }

        let file = tokio::fs::File::open(path).await?;

        Self::from_reader(file, opts).await.map(Some)
    }

    /// Load metadata from directory
    #[cfg_attr(feature = "doc-cfg", doc(cfg(any(feature = "json", feature = "cbor"))))]
    pub async fn from_dir(path: impl AsRef<Path>, opts: &DataOptions) -> Result<Option<Self>> {
        Self::from_file(path.as_ref().join(opts.file_name()), opts).await

        /*
        let path = path.as_ref();

        #[allow(clippy::single_element_loop)]
        for (name, opts) in &[
            #[cfg(feature = "json")]
            (
                concat!(env!("CARGO_PKG_NAME"), ".json"),
                DataOptions {
                    coding: DataCoding::Json,
                    compress: DataCompress::No,
                },
            ),
            #[cfg(feature = "cbor")]
            (
                concat!(env!("CARGO_PKG_NAME"), ".cbor"),
                DataOptions {
                    coding: DataCoding::Cbor,
                    compress: DataCompress::No,
                },
            ),
            #[cfg(all(feature = "json", feature = "lz4"))]
            (
                concat!(env!("CARGO_PKG_NAME"), ".json.lz4"),
                DataOptions {
                    coding: DataCoding::Json,
                    compress: DataCompress::Lz4,
                },
            ),
            #[cfg(all(feature = "cbor", feature = "lz4"))]
            (
                concat!(env!("CARGO_PKG_NAME"), ".cbor.lz4"),
                DataOptions {
                    coding: DataCoding::Cbor,
                    compress: DataCompress::Lz4,
                },
            ),
        ] {
            return match Self::from_file(path.join(name), opts).await {
                Ok(Some(res)) => Ok(Some(res)),
                Err(err) => Err(err),
                _ => continue,
            };
        }

        Ok(None)
        */
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
    pub async fn to_file(&self, path: impl AsRef<Path>, opts: &DataOptions) -> Result<()> {
        let mut file = tokio::fs::File::create(path).await?;

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
        self.to_file(path.as_ref().join(opts.file_name()), opts)
            .await
    }
}
