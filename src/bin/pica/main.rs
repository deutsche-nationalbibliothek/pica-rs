extern crate clap;
extern crate csv;
extern crate regex;
extern crate serde;
extern crate termcolor;

// mod cli;
mod commands;
// mod common;
mod config;
mod macros;
// mod translit;
mod util;

use std::path::PathBuf;
use std::{io, process};

use clap::{Parser, Subcommand};
use commands::{Cat, Count};
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

    /// Count records, fields and subfields
    Count(Count),
}

fn run() -> CliResult<()> {
    let args = Cli::parse();
    let config = Config::from_path_or_default(args.config)?;

    match args.command {
        Commands::Cat(cat) => cat.run(&config),
        Commands::Count(count) => count.run(&config),
    }

    // let mut app = cli::build_cli();
    // let m = app.clone().get_matches();
    // let name = m.subcommand_name().unwrap();
    // let args = m.subcommand_matches(name).unwrap();

    // match name {
    //     "cat" => cmds::cat::run(args, &config),
    //     "completions" => cmds::completions::run(args, &mut app),
    //     "count" => cmds::count::run(args, &config),
    //     "filter" => cmds::filter::run(args, &config),
    //     "frequency" => cmds::frequency::run(args, &config),
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
