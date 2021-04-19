use std::convert::From;
use std::{fmt, io};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Pica(pica::Error),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref e) => e.fmt(f),
            CliError::Pica(ref e) => e.fmt(f),
            CliError::Other(ref s) => f.write_str(&**s),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<pica::Error> for CliError {
    fn from(err: pica::Error) -> CliError {
        CliError::Pica(err)
    }
}
