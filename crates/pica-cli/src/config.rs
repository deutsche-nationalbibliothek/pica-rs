use std::path::{Path, PathBuf};
use std::{fs, io};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Config {
    /// The path of the config.
    #[serde(skip)]
    path: PathBuf,

    // Whether to skip invalid records or not.
    pub(crate) skip_invalid: bool,

    /// If set, a filter expression is translitered into the given
    /// unicode normalization form before applied on a record.
    pub(crate) normalization: Option<NormalizationForm>,

    /// This structure should always be constructed using a public
    /// constructor or using the update syntax:
    ///
    /// ```ignore
    /// use crate::config::Config;
    ///
    /// let config = Config {
    ///     ..Default::default()
    /// };
    /// ```
    #[doc(hidden)]
    #[serde(skip)]
    __non_exhaustive: (),
}

impl Config {
    /// Creates a new default config.
    pub(crate) fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().into(),
            ..Default::default()
        }
    }

    /// Loads a config from a path.
    pub(crate) fn from_path<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().into();
        let content = fs::read_to_string(&path)?;
        let mut config: Self =
            toml::from_str(&content).expect("valid config");
        config.path = path;

        Ok(config)
    }

    /// Creates a new configuration. The file location is derived from
    /// the standard directories and the name of the project and
    /// organization.
    pub(crate) fn discover() -> io::Result<Self> {
        if let Some(project_dir) =
            ProjectDirs::from("de.dnb", "DNB", "pica")
        {
            let config_dir = project_dir.config_dir();
            let config = config_dir.join("config.toml");

            if config.is_file() {
                Self::from_path(config)
            } else {
                Ok(Self::new(config))
            }
        } else {
            Ok(Self::default())
        }
    }

    /// Saves the config.
    pub(crate) fn save(&self) -> io::Result<()> {
        use std::fs::{File, create_dir_all};
        use std::io::Write;

        if let Some(parent) = self.path.parent()
            && !parent.is_dir()
        {
            create_dir_all(parent)?;
        }

        let content = toml::to_string(self).unwrap();
        let mut out = File::create(&self.path)?;
        out.write_all(content.as_bytes())?;
        Ok(())
    }
}
