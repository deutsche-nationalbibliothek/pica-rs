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
use commands::{Cat, Completions, Count, Filter, Frequency, Invalid};
use config::Config;
use util::{CliError, CliResult};

#[derive(Debug, Parser)]
#[clap(version, author, infer_subcommands = true)]
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
    /// Concatenate records from multiple files
    Cat(Cat),

    /// Generate shell completions (e.g. Bash, ZSH)
    Completions(Completions),

    /// Count records, fields and subfields
    Count(Count),

    /// Filter records by whether the given query matches
    Filter(Filter),

    /// Compute a frequency table of a subfield
    Frequency(Frequency),

    /// Filter out invalid records, which can't be decoded
    Invalid(Invalid),
}

fn run() -> CliResult<()> {
    let args = Cli::parse();
    let config = Config::from_path_or_default(args.config)?;

    match args.command {
        Commands::Cat(cmd) => cmd.run(&config),
        Commands::Count(cmd) => cmd.run(&config),
        Commands::Completions(cmd) => cmd.run(&mut Cli::command()),
        Commands::Filter(cmd) => cmd.run(&config),
        Commands::Frequency(cmd) => cmd.run(&config),
        Commands::Invalid(cmd) => cmd.run(),
    }

    // let mut app = cli::build_cli();
    // let m = app.clone().get_matches();
    // let name = m.subcommand_name().unwrap();
    // let args = m.subcommand_matches(name).unwrap();

    // match name {
    //     "invalid" => cmds::invalid::run(args),
    //     "json" => cmds::json::run(args, &config),
    //     "partition" => cmds::partition::run(args, &config),
    //     "print" => cmds::print::run(args, &config),
    //     "sample" => cmds::sample::run(args, &config),
    //     "select" => cmds::select::run(args, &config),
    //     "slice" => cmds::slice::run(args, &config),
    //     "split" => cmds::split::run(args, &config),
    //     "xml" => cmds::xml::run(args, &config),
    //     _ => unreachable!(),
    // }
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
        Err(CliError::Pica(err)) => {
            eprintln!("Pica Error: {}", err);
            process::exit(1);
        }
        Err(CliError::Io(err)) => {
            eprintln!("IO Error: {}", err);
            process::exit(1);
        }
        Err(CliError::Csv(err)) => {
            eprintln!("CSV Error: {}", err);
            process::exit(1);
        }
        Err(CliError::Xml(err)) => {
            eprintln!("XML Error: {}", err);
            process::exit(1);
        }
        Err(CliError::Other(err)) => {
            eprintln!("error: {}", err);
            process::exit(1);
        }
    }
}
