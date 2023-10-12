use std::str::FromStr;

use thiserror::Error;

/// The level (main, local, copy) of a field (or tag).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Level {
    #[default]
    Main,
    Local,
    Copy,
}

/// An error that can occur when parsing PICA+ level.
#[derive(Error, PartialEq, Eq, Debug)]
#[error("{0}")]
pub struct ParseLevelError(String);

impl FromStr for Level {
    type Err = ParseLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "main" => Ok(Self::Main),
            "local" => Ok(Self::Local),
            "copy" => Ok(Self::Copy),
            _ => Err(ParseLevelError(format!("invalid level '{}'", s))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() -> anyhow::Result<()> {
        assert_eq!("main".parse::<Level>().unwrap(), Level::Main);
        assert_eq!("local".parse::<Level>().unwrap(), Level::Local);
        assert_eq!("copy".parse::<Level>().unwrap(), Level::Copy);

        assert!("master".parse::<Level>().is_err());
        assert!("foo".parse::<Level>().is_err());

        Ok(())
    }
}
