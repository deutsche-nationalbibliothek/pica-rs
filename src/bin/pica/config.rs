use std::fs::{create_dir_all, read_to_string};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GlobalConfig {
    pub(crate) skip_invalid: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(skip)]
    pub(crate) path: Option<PathBuf>,
    pub(crate) global: Option<GlobalConfig>,
    pub(crate) cat: Option<crate::cmds::cat::CatConfig>,
    pub(crate) count: Option<crate::cmds::count::CountConfig>,
    pub(crate) filter: Option<crate::cmds::filter::FilterConfig>,
    pub(crate) frequency:
        Option<crate::cmds::frequency::FrequencyConfig>,
    pub(crate) json: Option<crate::cmds::json::JsonConfig>,
    pub(crate) partition:
        Option<crate::cmds::partition::PartitionConfig>,
    pub(crate) print: Option<crate::cmds::print::PrintConfig>,
    pub(crate) sample: Option<crate::cmds::sample::SampleConfig>,
    pub(crate) select: Option<crate::cmds::select::SelectConfig>,
    pub(crate) slice: Option<crate::cmds::slice::SliceConfig>,
    pub(crate) split: Option<crate::cmds::split::SplitConfig>,
    pub(crate) xml: Option<crate::cmds::xml::XmlConfig>,
}

impl Config {
    pub(crate) fn new() -> Result<Self, std::io::Error> {
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

    pub(crate) fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<Self, std::io::Error> {
        let path = PathBuf::from(path.as_ref());
        let content = read_to_string(&path)?;

        // FIXME: handle unwrap()
        let mut config: Config = toml::from_str(&content).unwrap();
        config.path = Some(path);

        Ok(config)
    }

    pub(crate) fn from_path_or_default<P: AsRef<Path>>(
        path: Option<P>,
    ) -> Result<Self, std::io::Error> {
        match path {
            Some(path) => Self::from_path(path),
            None => Self::new(),
        }
    }
}
