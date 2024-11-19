use clap::{Parser, Subcommand};

use crate::commands::*;

/// pica is a fast command-line tool to process bibliographic records
/// encoded in PICA+.
#[derive(Debug, Parser)]
#[command(name = "pica", version, about, long_about = None)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) cmd: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Invalid(Invalid),
}
