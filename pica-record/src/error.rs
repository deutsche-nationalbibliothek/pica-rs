use thiserror::Error;

/// An error that can occur when ceparsing PICA+ records.

/// -----{ TODO }-----------------------------------------
/// An error that can occur when parsing PICA+ records.
#[derive(Error, PartialEq, Eq, Debug)]
pub enum ParsePicaError {
    #[error("invalid subfield")]
    InvalidSubfield,
    #[error("invalid tag")]
    InvalidTag,
    #[error("invalid occurrence")]
    InvalidOccurrence,
    #[error("invalid field")]
    InvalidField,
    #[error("invalid record (expected valid record, got `{0:?}`)")]
    InvalidRecord(Vec<u8>),
}
