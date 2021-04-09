use crate::parser::ParsePicaError;

use std::error;
use std::fmt::{self, Display, Formatter};

/// A type alias for `Result<T, pica::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur when processing PICA+ data.
#[derive(Debug)]
pub enum Error {
    InvalidSubfield(String),
    InvalidOccurrence(String),
    InvalidField(String),
    InvalidRecord(ParsePicaError),
    Utf8Error(std::str::Utf8Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::InvalidSubfield(ref m) => f.write_str(m),
            Error::InvalidOccurrence(ref m) => f.write_str(m),
            Error::InvalidField(ref m) => f.write_str(m),
            Error::InvalidRecord(ref e) => e.fmt(f),
            Error::Utf8Error(ref e) => e.fmt(f),
        }
    }
}

impl From<ParsePicaError> for Error {
    fn from(err: ParsePicaError) -> Self {
        Self::InvalidRecord(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}
