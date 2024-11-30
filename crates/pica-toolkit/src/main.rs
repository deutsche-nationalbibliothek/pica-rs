extern crate clap;
extern crate csv;
extern crate regex;
extern crate serde;
extern crate termcolor;

mod commands;
mod config;
mod error;
mod filter_list;
mod macros;
mod progress;

use std::path::PathBuf;
use std::{io, process};

use clap::{Parser, Subcommand};
use commands::{
    Convert, Explode, Filter, Frequency, Hash, Partition, Print,
    Sample, Select, Slice, Split,
};
use config::Config;
use error::{CliError, CliResult};

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
    Convert(Convert),
    Explode(Explode),
    Filter(Filter),
    Frequency(Frequency),
    Hash(Hash),
    Partition(Partition),
    Print(Print),
    Sample(Sample),
    Select(Select),
    Slice(Slice),
    Split(Split),
}

fn run() -> CliResult<()> {
    let args = Cli::parse();
    let config = Config::from_path_or_default(args.config)?;

    match args.command {
        Commands::Convert(cmd) => cmd.run(&config),
        Commands::Explode(cmd) => cmd.run(&config),
        Commands::Filter(cmd) => cmd.run(&config),
        Commands::Frequency(cmd) => cmd.run(&config),
        Commands::Hash(cmd) => cmd.run(&config),
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
        Err(CliError::ParsePica(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
        Err(CliError::ReadPica(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
        Err(CliError::ParsePath(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
        Err(CliError::ParseMatcher(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
        Err(CliError::ParseQuery(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
        Err(CliError::Io(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
        Err(CliError::Csv(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
        Err(CliError::Other(err)) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}
