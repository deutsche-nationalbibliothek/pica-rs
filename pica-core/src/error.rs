use std::error::Error;
use std::fmt::Display;

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidTag,
    InvalidOccurrence,
    InvalidSubfield,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("parse error: ")?;

        match *self {
            Self::InvalidTag => f.write_str("invalid tag"),
            Self::InvalidOccurrence => f.write_str("invalid occurrence"),
            Self::InvalidSubfield => f.write_str("invalid subfield"),
        }
    }
}
