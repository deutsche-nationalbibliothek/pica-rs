use std::error::Error;
use std::fmt::{self, Display};

/// An error that can occur when parsing matcher expressions.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidOccurrenceMatcher,
    InvalidTagMatcher,
    InvalidSubfieldMatcher,
    InvalidSubfieldListMatcher,
    InvalidFieldMatcher,
    InvalidRecordMatcher,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("parse error: invalid ")?;

        match *self {
            Self::InvalidFieldMatcher => f.write_str("tag matcher"),
            Self::InvalidOccurrenceMatcher => f.write_str("occurrence matcher"),
            Self::InvalidRecordMatcher => f.write_str("record matcher"),
            Self::InvalidSubfieldListMatcher => f.write_str("subfield matcher"),
            Self::InvalidSubfieldMatcher => f.write_str("subfield matcher"),
            Self::InvalidTagMatcher => f.write_str("tag matcher"),
        }
    }
}
