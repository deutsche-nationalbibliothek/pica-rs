use std::fmt::{self, Display};

use serde::Deserialize;

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    #[default]
    Error,
    Warning,
    Info,
}

impl Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        };

        write!(f, "{level_str}")
    }
}
