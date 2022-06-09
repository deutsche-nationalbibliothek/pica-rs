use crate::parser::{ParsePathError, ParsePicaError};

use std::fmt::{self, Display, Formatter};
use std::{error, io};

/// A type alias for `Result<T, pica::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur when processing PICA+ data.
#[derive(Debug)]
pub enum Error {
    InvalidTag(String),
    InvalidOccurrence(String),
    InvalidOccurrenceMatcher(String),
    InvalidSubfield(String),
    InvalidSubfieldMatcher(String),
    InvalidSubfieldsMatcher(String),
    InvalidField(String),
    InvalidFieldMatcher(String),
    InvalidRecord(ParsePicaError),
    InvalidPath(ParsePathError),
    InvalidMatcher(String),
    Utf8Error(std::str::Utf8Error),
    Io(io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Self::InvalidTag(ref m) => f.write_str(m),
            Self::InvalidOccurrence(ref m) => f.write_str(m),
            Self::InvalidOccurrenceMatcher(ref m) => f.write_str(m),
            Self::InvalidSubfield(ref m) => f.write_str(m),
            Self::InvalidSubfieldMatcher(ref m) => f.write_str(m),
            Self::InvalidSubfieldsMatcher(ref m) => f.write_str(m),
            Self::InvalidField(ref m) => f.write_str(m),
            Self::InvalidFieldMatcher(ref m) => f.write_str(m),
            Self::InvalidMatcher(ref m) => f.write_str(m),
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

impl From<pica_core::ParseError> for Error {
    fn from(err: pica_core::ParseError) -> Self {
        match err {
            pica_core::ParseError::InvalidTag => {
                Self::InvalidTag("invalid tag".to_string())
            }
        }
    }
}
