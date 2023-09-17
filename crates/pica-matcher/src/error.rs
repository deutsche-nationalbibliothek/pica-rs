use thiserror::Error;

/// An error that can occur when parsing matcher expressions.
#[derive(Error, PartialEq, Eq, Debug)]
pub enum ParseMatcherError {
    #[error("invalid tag matcher")]
    InvalidTagMatcher,
    #[error("invalid occurrence matcher (got `{0}`)")]
    InvalidOccurrenceMatcher(String),
    #[error("invalid subfield matcher (got `{0}`)")]
    InvalidSubfieldMatcher(String),
    #[error("invalid field matcher (got `{0}`)")]
    InvalidFieldMatcher(String),
    #[error("invalid record matcher (got `{0}`)")]
    InvalidRecordMatcher(String),
}
