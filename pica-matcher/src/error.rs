use thiserror::Error;

/// An error that can occur when parsing matcher expressions.
#[derive(Error, PartialEq, Eq, Debug)]
pub enum ParseMatcherError {
    #[error("invalid tag matcher")]
    InvalidTagMatcher,
}