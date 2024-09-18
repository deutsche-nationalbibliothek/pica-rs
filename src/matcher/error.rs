use thiserror::Error;

/// An error that can occur when parsing PICA+ matcher.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct ParseMatcherError(pub(crate) String);
