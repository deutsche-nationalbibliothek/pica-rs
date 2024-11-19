use std::fs::{self, create_dir_all, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Config {
    /// The path of the config.
    #[serde(skip)]
    path: PathBuf,

    // Whether to skip invalid records or not.
    pub(crate) skip_invalid: bool,

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
    /// Creates a new default config and sets the file location.
    pub(crate) fn discover() -> io::Result<Self> {
        if let Some(project_dir) =
            ProjectDirs::from("de.dnb", "DNB", "pica")
        {
            let config_dir = project_dir.config_dir();
            let config = config_dir.join("config.toml");

            if config.is_file() {
                Self::from_path(config)
            } else {
                Ok(Self {
                    path: config,
                    ..Default::default()
                })
            }
        } else {
            Ok(Self::default())
        }
    }

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

    /// Saves the config.
    #[allow(dead_code)]
    pub(crate) fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.is_dir() {
                create_dir_all(parent)?;
            }
        }

        let content = toml::to_string(self).unwrap();
        let mut out = File::create(&self.path)?;
        out.write_all(content.as_bytes())?;
        Ok(())
    }
}
