use std::{convert::From, fmt, io};

pub type App = clap::App<'static, 'static>;
pub type CliArgs = clap::ArgMatches<'static>;
pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref e) => e.fmt(f),
            CliError::Other(ref s) => f.write_str(&**s),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}
