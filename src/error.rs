use crate::ParsePicaError;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;

/// A type alias for `Result<T, pica::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur when processing PICA+ data.
#[derive(Debug)]
pub enum Error {
    InvalidRecord(ParsePicaError),
    Io(io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::InvalidRecord(ref e) => e.fmt(f),
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
