use crate::matcher::ParseMatcherError;
use crate::path::ParsePathError;
use crate::primitives::ParsePicaError;

/// An error that can occur in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ParsePica(ParsePicaError),
    #[error(transparent)]
    ParseMatcher(ParseMatcherError),
    #[error(transparent)]
    ParsePath(ParsePathError),
    #[error(transparent)]
    ParseFormat(crate::fmt::ParseFormatError),
    #[error(transparent)]
    ParseQuery(crate::query::ParseQueryError),
}
