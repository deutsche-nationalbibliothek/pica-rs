use crate::util::{CliError, CliResult};
use directories::ProjectDirs;

#[derive(Debug)]
pub struct Config {
    config: config::Config,
}

impl Config {
    /// Create a new `Config`
    ///
    /// # Rules
    ///
    /// - if a filename is provided, the settings are taken from this file
    /// - if no filename is provided and a file is found in the application
    ///   directory, the settings are taken from this file
    /// - otherwise, an empty, default configuration is returned
    pub fn new(filename: Option<&str>) -> CliResult<Config> {
        let mut config = config::Config::default();

        if let Some(filename) = filename {
            if let Err(err) = config.merge(config::File::with_name(filename)) {
                return Err(CliError::Config(err.to_string()));
            }
        } else if let Some(proj_dirs) =
            ProjectDirs::from("de.dnb", "Deutsche Nationalbibliothek", "pica")
        {
            let user_config = proj_dirs.config_dir().join("Pica.toml");
            if user_config.is_file() {
                let result = config.merge(config::File::with_name(
                    user_config.to_str().unwrap(),
                ));

                if let Err(err) = result {
                    return Err(CliError::Config(err.to_string()));
                }
            }
        }

        Ok(Self { config })
    }

    pub fn get_bool(
        &self,
        section: &str,
        key: &str,
        global: bool,
    ) -> Result<bool, config::ConfigError> {
        let mut retval = self.config.get_bool(&format!("{}.{}", section, key));
        if global && retval.is_err() {
            retval = self.config.get_bool(&format!("global.{}", key));
        }

        retval
    }

    pub fn get_string(
        &self,
        section: &str,
        key: &str,
        global: bool,
    ) -> Result<String, config::ConfigError> {
        let mut retval = self.config.get_str(&format!("{}.{}", section, key));
        if global && retval.is_err() {
            retval = self.config.get_str(&format!("global.{}", key));
        }

        retval
    }
}
