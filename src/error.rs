use crate::parser::{ParsePathError, ParsePicaError};

use std::fmt::{self, Display, Formatter};
use std::{error, io};

/// A type alias for `Result<T, pica::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur when processing PICA+ data.
#[derive(Debug)]
pub enum Error {
    InvalidSubfield(String),
    InvalidOccurrence(String),
    InvalidField(String),
    InvalidRecord(ParsePicaError),
    InvalidPath(ParsePathError),
    Utf8Error(std::str::Utf8Error),
    Io(io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::InvalidSubfield(ref m) => f.write_str(m),
            Error::InvalidOccurrence(ref m) => f.write_str(m),
            Error::InvalidField(ref m) => f.write_str(m),
            Error::InvalidRecord(ref e) => e.fmt(f),
            Error::InvalidPath(ref e) => e.fmt(f),
            Error::Utf8Error(ref e) => e.fmt(f),
            Error::Io(ref e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ParsePicaError> for Error {
    fn from(err: ParsePicaError) -> Self {
        Self::InvalidRecord(err)
    }
}

impl From<ParsePathError> for Error {
    fn from(err: ParsePathError) -> Self {
        Self::InvalidPath(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}
