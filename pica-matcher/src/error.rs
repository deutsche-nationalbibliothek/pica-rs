use std::error::Error;
use std::fmt::{self, Display};

/// An error that can occur when parsing matcher expressions.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidOccurrenceMatcher,
    InvalidTagMatcher,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("parse error: ")?;

        match *self {
            Self::InvalidOccurrenceMatcher => {
                f.write_str("invalid occurrence matcher")
            }
            Self::InvalidTagMatcher => f.write_str("invalid tag matcher"),
        }
    }
}
