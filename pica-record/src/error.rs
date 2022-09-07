use thiserror::Error;

/// An error that can occur when parsing PICA+ records.
#[derive(Error, Debug)]
pub enum ParsePicaError {
    #[error("invalid subfield")]
    InvalidSubfield,
}
