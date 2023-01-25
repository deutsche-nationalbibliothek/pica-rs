use std::convert::From;
use std::{fmt, io};

pub(crate) type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub(crate) enum CliError {
    Io(io::Error),
    Csv(csv::Error),
    Xml(xml::writer::Error),
    Pica(pica::Error),
    ParsePica(pica_record::ParsePicaError),
    ParsePath(pica_path::ParsePathError),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Csv(ref e) => e.fmt(f),
            CliError::Xml(ref e) => e.fmt(f),
            CliError::Io(ref e) => e.fmt(f),
            CliError::Pica(ref e) => e.fmt(f),
            CliError::ParsePica(ref e) => e.fmt(f),
            CliError::ParsePath(ref e) => e.fmt(f),
            CliError::Other(ref s) => f.write_str(s),
        }
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

impl From<pica_record::io::ReadPicaError> for CliError {
    fn from(err: pica_record::io::ReadPicaError) -> Self {
        match err {
            pica_record::io::ReadPicaError::Io(e) => CliError::Io(e),
            pica_record::io::ReadPicaError::Parse(e) => {
                CliError::ParsePica(e)
            }
        }
    }
}

impl From<pica_path::ParsePathError> for CliError {
    fn from(err: pica_path::ParsePathError) -> Self {
        CliError::ParsePath(err)
    }
}
