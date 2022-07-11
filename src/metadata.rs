use crate::{Path, PathBuf};
use std::collections::{BTreeMap as Map, BTreeSet as Set};

/// Data associated with source file
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceData {
    /// Configuration options associated with source file
    #[cfg_attr(feature = "serde", serde(rename = "o"))]
    pub config_opts: Set<String>,

    /// Compatible strings of source file
    #[cfg_attr(feature = "serde", serde(rename = "s"))]
    pub compat_strs: Set<String>,

    /// Module data
    #[cfg_attr(feature = "serde", serde(rename = "m"))]
    pub module: Option<ModuleData>,
}

/// Data associated with module
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModuleData {
    /// Module authors
    #[cfg_attr(feature = "serde", serde(rename = "a"))]
    pub authors: Vec<String>,

    /// Module description
    #[cfg_attr(feature = "serde", serde(rename = "d"))]
    pub description: String,

    /// Module license
    #[cfg_attr(feature = "serde", serde(rename = "l"))]
    pub license: String,

    /// Module aliases
    #[cfg_attr(feature = "serde", serde(rename = "s"))]
    pub aliases: Vec<String>,

    /// Module parameters
    #[cfg_attr(feature = "serde", serde(rename = "p"))]
    pub params: Map<String, ParamData>,
}

/// Data associated with parameter
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParamData {
    /// Parameter type
    #[cfg_attr(feature = "serde", serde(rename = "t"))]
    pub type_: String,

    /// Parameter permissions
    #[cfg_attr(feature = "serde", serde(rename = "p"))]
    pub perm: u16,

    /// Parameter description
    #[cfg_attr(feature = "serde", serde(rename = "d"))]
    pub description: String,
}

/// Data related to configuration option
#[derive(Debug, Default, Clone)]
pub struct ConfigOptData {
    /// Source files related to configuration option
    pub sources: Set<PathBuf>,
}

/// Data associated with compatible string
#[derive(Debug, Default, Clone)]
pub struct CompatStrData {
    /// Source file associated with compatible string
    pub source: PathBuf,
}

/// Source-code metadata
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MetaData {
    /// Data associated with source files
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub sources: Map<PathBuf, SourceData>,

    /// Data associated with configuration options
    #[cfg_attr(feature = "serde", serde(skip))]
    pub config_opts: Map<String, ConfigOptData>,

    /// Data associated with compatible strings
    #[cfg_attr(feature = "serde", serde(skip))]
    pub compat_strs: Map<String, CompatStrData>,
}

impl MetaData {
    /// Get reference to source data by path
    pub fn source(&self, source: impl AsRef<Path>) -> Option<&SourceData> {
        self.sources.get(source.as_ref())
    }

    /// Get mutable reference to source data by path
    pub fn source_mut(&mut self, source: impl AsRef<Path>) -> &mut SourceData {
        let source = source.as_ref();

        if !self.sources.contains_key(source) {
            self.sources.insert(source.into(), Default::default());
        }

        self.sources.get_mut(source).unwrap()
    }

    /// Get reference to configuration option data by name
    pub fn config_opt(&self, config_opt: impl AsRef<str>) -> Option<&ConfigOptData> {
        self.config_opts.get(config_opt.as_ref())
    }

    /// Get mutable reference to configuration option data by name
    pub fn config_opt_mut(&mut self, config_opt: impl AsRef<str>) -> &mut ConfigOptData {
        let config_opt = config_opt.as_ref();

        if !self.config_opts.contains_key(config_opt) {
            self.config_opts
                .insert(config_opt.into(), Default::default());
        }

        self.config_opts.get_mut(config_opt).unwrap()
    }

    /// Get reference to compatible string data by string
    pub fn compat_str(&self, compat_str: impl AsRef<str>) -> Option<&CompatStrData> {
        self.compat_strs.get(compat_str.as_ref())
    }

    /// Get mutable reference to compatible string data by string
    pub fn compat_str_mut(&mut self, compat_str: impl AsRef<str>) -> &mut CompatStrData {
        let compat_str = compat_str.as_ref();

        if !self.compat_strs.contains_key(compat_str) {
            self.compat_strs
                .insert(compat_str.into(), Default::default());
        }

        self.compat_strs.get_mut(compat_str).unwrap()
    }

    /// Synchronize all data with sources data
    pub fn sync_with_sources(&mut self) {
        let mut this = Self::default();

        for (
            source,
            SourceData {
                config_opts,
                compat_strs,
                ..
            },
        ) in &self.sources
        {
            for config_opt in config_opts {
                this.config_opt_mut(config_opt).add_source(&source);
            }
            for compat_str in compat_strs {
                this.compat_str_mut(compat_str).set_source(&source);
            }
        }

        self.config_opts = this.config_opts;
        self.compat_strs = this.compat_strs;
    }
}

impl SourceData {
    /// Add related configuration options to source data
    pub fn add_config_opts(&mut self, config_opts: impl Into<Set<String>>) {
        self.config_opts.extend(config_opts.into());
    }

    /// Add associated compatible strings to source data
    pub fn add_compat_strs(&mut self, compat_strs: impl Into<Set<String>>) {
        self.compat_strs.extend(compat_strs.into());
    }
}

impl ModuleData {
    /// Check that all module fields is empty
    pub fn is_empty(&self) -> bool {
        self.authors.is_empty()
            && self.description.is_empty()
            && self.license.is_empty()
            && self.aliases.is_empty()
            && self.params.is_empty()
    }
}

impl ConfigOptData {
    /// Add related source to configuration option data
    pub fn add_source(&mut self, source: impl Into<PathBuf>) {
        self.sources.insert(source.into());
    }
}

impl CompatStrData {
    /// Set associated source to compatible string data
    pub fn set_source(&mut self, source: impl Into<PathBuf>) {
        self.source = source.into();
    }
}
