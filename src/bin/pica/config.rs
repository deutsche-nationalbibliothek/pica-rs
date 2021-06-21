use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_to_string};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    pub skip_invalid: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    pub global: Option<GlobalConfig>,
    pub cat: Option<crate::cmds::cat::CatConfig>,
    pub filter: Option<crate::cmds::filter::FilterConfig>,
    pub frequency: Option<crate::cmds::frequency::FrequencyConfig>,
    pub json: Option<crate::cmds::json::JsonConfig>,
    pub partition: Option<crate::cmds::partition::PartitionConfig>,
    pub print: Option<crate::cmds::print::PrintConfig>,
}

impl Config {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut config = Config::default();

        if let Some(project_dirs) =
            ProjectDirs::from("de.dnb", "DNB", "pica-rs")
        {
            let config_dir = project_dirs.config_dir();
            if !config_dir.exists() {
                create_dir_all(config_dir)?;
            }

            let config_file = config_dir.join("Pica.toml");
            if config_file.exists() {
                return Self::from_path(config_file);
            }

            config.path = Some(config_file);
        }

        Ok(config)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = PathBuf::from(path.as_ref());
        let content = read_to_string(&path)?;

        // FIXME: handle unwrap()
        let mut config: Config = toml::from_str(&content).unwrap();
        config.path = Some(path);

        Ok(config)
    }

    pub fn from_path_or_default<P: AsRef<Path>>(
        path: Option<P>,
    ) -> Result<Self, std::io::Error> {
        match path {
            Some(path) => Self::from_path(path),
            None => Self::new(),
        }
    }
}
