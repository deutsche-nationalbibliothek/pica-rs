#[macro_use]
extern crate clap;
extern crate csv;
extern crate directories;
extern crate regex;

mod cli;
mod cmds;
mod config;
mod util;

use crate::config::Config;
use std::{io, process};
use util::CliError;

fn main() {
    let mut app = cli::build_cli();
    let m = app.clone().get_matches();
    let name = m.subcommand_name().unwrap();
    let args = m.subcommand_matches(name).unwrap();

    let config = match Config::new(m.value_of("config")) {
        Err(err) => {
            eprintln!("config: {}", err);
            process::exit(1);
        }
        Ok(config) => config,
    };

    let result = match name {
        "cat" => cmds::cat::run(args, &config),
        "completion" => cmds::completion::run(args, &mut app),
        "filter" => cmds::filter::run(args, &config),
        "frequency" => cmds::frequency::run(args, &config),
        "invalid" => cmds::invalid::run(args),
        "json" => cmds::json::run(args, &config),
        "partition" => cmds::partition::run(args, &config),
        "print" => cmds::print::run(args, &config),
        "sample" => cmds::sample::run(args, &config),
        "select" => cmds::select::run(args, &config),
        "slice" => cmds::slice::run(args, &config),
        "split" => cmds::split::run(args, &config),
        _ => unreachable!(),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(CliError::Io(ref err))
            if err.kind() == io::ErrorKind::BrokenPipe =>
        {
            process::exit(0);
        }
        Err(CliError::Pica(pica::Error::Io(ref err)))
            if err.kind() == io::ErrorKind::BrokenPipe =>
        {
            process::exit(0);
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
        Err(CliError::Config(err)) => {
            eprintln!("config: {}", err);
            process::exit(1);
        }
        Err(CliError::Other(err)) => {
            eprintln!("error: {}", err);
            process::exit(1);
        }
    }
}
