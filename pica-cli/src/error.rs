use std::process::ExitCode;

use pica_record::io::ReadPicaError;
use thiserror::Error;

pub(crate) type CliResult = Result<ExitCode, CliError>;

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error(transparent)]
    ReadPica(#[from] ReadPicaError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
