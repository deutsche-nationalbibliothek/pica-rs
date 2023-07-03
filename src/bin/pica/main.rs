extern crate clap;
extern crate csv;
extern crate regex;
extern crate serde;
extern crate termcolor;

// mod cli;
mod commands;
mod common;
mod config;
mod macros;
mod translit;
mod util;

use std::path::PathBuf;
use std::{io, process};

use clap::{CommandFactory, Parser, Subcommand};
use commands::{
    Cat, Completions, Convert, Count, Filter, Frequency, Hash, Invalid,
    Json, Partition, Print, Sample, Select, Slice, Split,
};
use config::Config;
use util::{CliError, CliResult};

#[derive(Debug, Parser)]
#[clap(version, author, infer_subcommands = true, max_term_width = 72)]
#[command(name = "pica")]
#[command(
    about = "Tools to work with bibliographic records encoded in PICA+."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Cat(Cat),

    /// Generate shell completions (e.g. Bash, ZSH)
    Completions(Completions),
    Convert(Convert),
    Count(Count),
    Filter(Filter),
    Frequency(Frequency),
    Hash(Hash),
    Invalid(Invalid),

    /// Serialize records to JSON
    Json(Json),
    Partition(Partition),
    Print(Print),

    /// Selects a random permutation of records
    Sample(Sample),

    /// Select subfield values from records
    Select(Select),
    Slice(Slice),
    Split(Split),
}

fn run() -> CliResult<()> {
    let args = Cli::parse();
    let config = Config::from_path_or_default(args.config)?;

    match args.command {
        Commands::Cat(cmd) => cmd.run(&config),
        Commands::Completions(cmd) => cmd.run(&mut Cli::command()),
        Commands::Convert(cmd) => cmd.run(&config),
        Commands::Count(cmd) => cmd.run(&config),
        Commands::Filter(cmd) => cmd.run(&config),
        Commands::Frequency(cmd) => cmd.run(&config),
        Commands::Hash(cmd) => cmd.run(&config),
        Commands::Invalid(cmd) => cmd.run(&config),
        Commands::Json(cmd) => cmd.run(&config),
        Commands::Partition(cmd) => cmd.run(&config),
        Commands::Print(cmd) => cmd.run(&config),
        Commands::Sample(cmd) => cmd.run(&config),
        Commands::Select(cmd) => cmd.run(&config),
        Commands::Slice(cmd) => cmd.run(&config),
        Commands::Split(cmd) => cmd.run(&config),
    }
}

fn main() {
    match run() {
        Ok(()) => process::exit(0),
        Err(CliError::Io(ref err))
            if err.kind() == io::ErrorKind::BrokenPipe =>
        {
            process::exit(0); // no-coverage
        }
        Err(CliError::Pica(pica::Error::Io(ref err)))
            if err.kind() == io::ErrorKind::BrokenPipe =>
        {
            process::exit(0); // no-coverage
        }
        Err(CliError::ParsePica(err)) => {
            eprintln!("Parse Pica Error: {err}");
            process::exit(1);
        }
        Err(CliError::ParsePath(err)) => {
            eprintln!("Parse Path Error: {err}");
            process::exit(1);
        }
        Err(CliError::ParseMatcher(err)) => {
            eprintln!("Parse Matcher Error: {err}");
            process::exit(1);
        }
        Err(CliError::ParseQuery(err)) => {
            eprintln!("Parse Query Error: {err}");
            process::exit(1);
        }
        Err(CliError::Pica(err)) => {
            eprintln!("Pica Error: {err}");
            process::exit(1);
        }
        Err(CliError::Io(err)) => {
            eprintln!("IO Error: {err}");
            process::exit(1);
        }
        Err(CliError::Csv(err)) => {
            eprintln!("CSV Error: {err}");
            process::exit(1);
        }
        Err(CliError::Other(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}
