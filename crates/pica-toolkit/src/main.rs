extern crate clap;
extern crate csv;
extern crate regex;
extern crate serde;

mod commands;
mod config;
mod error;
mod filter_list;
mod macros;
mod progress;

use std::path::PathBuf;
use std::{io, process};

use clap::{Parser, Subcommand};
use commands::{Convert, Filter};
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
#[allow(clippy::large_enum_variant)]
enum Commands {
    Convert(Convert),
    Filter(Filter),
}

fn run() -> CliResult<()> {
    let args = Cli::parse();
    let config = Config::from_path_or_default(args.config)?;

    match args.command {
        Commands::Convert(cmd) => cmd.run(&config),
        Commands::Filter(cmd) => cmd.run(&config),
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
