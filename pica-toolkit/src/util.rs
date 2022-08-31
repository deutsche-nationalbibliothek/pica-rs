use std::convert::From;
use std::{fmt, io};

pub(crate) type Command = clap::Command<'static>;
pub(crate) type CliArgs = clap::ArgMatches;
pub(crate) type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub(crate) enum CliError {
    Io(io::Error),
    Csv(csv::Error),
    Xml(xml::writer::Error),
    Pica(pica::Error),
    Matcher(pica_api::matcher::ParseError),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Csv(ref e) => e.fmt(f),
            CliError::Xml(ref e) => e.fmt(f),
            CliError::Io(ref e) => e.fmt(f),
            CliError::Pica(ref e) => e.fmt(f),
            CliError::Matcher(ref e) => e.fmt(f),
            CliError::Other(ref s) => f.write_str(s),
        }
    }
}

impl From<pica_api::matcher::ParseError> for CliError {
    fn from(e: pica_api::matcher::ParseError) -> CliError {
        CliError::Matcher(e)
    }
}

impl From<pica::Error> for CliError {
    fn from(err: pica::Error) -> CliError {
        CliError::Pica(err)
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<csv::Error> for CliError {
    fn from(err: csv::Error) -> CliError {
        CliError::Csv(err)
    }
}

impl From<xml::writer::Error> for CliError {
    fn from(err: xml::writer::Error) -> CliError {
        CliError::Xml(err)
    }
}
