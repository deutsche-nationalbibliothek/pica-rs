use std::error::Error;
use std::fmt::Display;

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidTag,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InvalidTag => f.write_str("pica parse error: invalid tag"),
        }
    }
}
