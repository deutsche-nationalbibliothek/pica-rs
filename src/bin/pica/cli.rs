use clap::AppSettings;
use std::{fmt, io};

pub type App = clap::App<'static>;

pub(crate) fn build_cli() -> App {
    App::new("pica")
        .about("Tools to work with bibliographic records encoded in Pica+")
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .author(crate_authors!())
        .subcommands(crate::commands::commands())
}

pub type CliResult<T> = Result<T, CliError>;
pub type CliArgs = clap::ArgMatches;

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Pica(pica::Error),
    // Other(String),
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

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref e) => e.fmt(f),
            CliError::Pica(ref e) => e.fmt(f),
            // CliError::Other(ref s) => f.write_str(&**s),
        }
    }
}
