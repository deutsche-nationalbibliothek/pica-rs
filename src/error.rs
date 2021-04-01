use crate::ParsePicaError;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::string::FromUtf8Error;

/// A type alias for `Result<T, pica::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur when processing PICA+ data.
#[derive(Debug)]
pub enum Error {
    InvalidField(String),
    InvalidOccurrence(String),
    InvalidRecord(ParsePicaError),
    InvalidSubfield(String),
    Utf8Error(FromUtf8Error),
    Io(io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::InvalidField(ref m) => f.write_str(m),
            Error::InvalidOccurrence(ref m) => f.write_str(m),
            Error::InvalidRecord(ref e) => e.fmt(f),
            Error::InvalidSubfield(ref m) => f.write_str(m),
            Error::Io(ref e) => e.fmt(f),
            Error::Utf8Error(ref e) => e.fmt(f),
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

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::Utf8Error(err)
    }
}
