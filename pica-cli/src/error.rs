use std::process::ExitCode;

use pica_record::io::ReadPicaError;
use pica_record::matcher::ParseMatcherError;
use thiserror::Error;

pub(crate) type CliResult = Result<ExitCode, CliError>;

#[cfg(feature = "unstable")]
macro_rules! bail {
    ($($arg:tt)*) => {{
        return Err(CliError::Other(format!($($arg)*)));
    }};
}

#[cfg(feature = "unstable")]
pub(crate) use bail;

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error(transparent)]
    ReadPica(#[from] ReadPicaError),
    #[error(transparent)]
    ParseMatcher(#[from] ParseMatcherError),
    #[error(transparent)]
    CSV(#[from] csv::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[cfg(feature = "unstable")]
    #[error("{0}")]
    Other(String),
}
