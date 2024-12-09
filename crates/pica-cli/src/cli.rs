use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands::*;

/// pica is a fast command-line tool to process bibliographic records
/// encoded in PICA+.
#[derive(Debug, Parser)]
#[clap(version, author, infer_subcommands = true, max_term_width = 72)]
#[command(name = "pica", version, about, long_about = None)]
pub(crate) struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub(crate) config: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) cmd: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Completions(Completions),
    Concat(Concat),
    #[cfg(feature = "unstable")]
    Config(Config),
    Convert(Convert),
    Count(Count),
    Explode(Explode),
    Filter(Filter),
    Frequency(Frequency),
    Hash(Hash),
    Invalid(Invalid),
    Partition(Partition),
    Print(Print),
    Sample(Sample),
    Select(Select),
    Slice(Slice),
    Split(Split),
}
