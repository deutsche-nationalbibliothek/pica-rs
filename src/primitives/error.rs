use thiserror::Error;

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct ParsePicaError(pub(crate) String);
