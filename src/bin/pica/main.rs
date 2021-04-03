#[macro_use]
extern crate clap;

mod cli;
mod commands;

use std::io;
use std::process;

use cli::{build_cli, CliError};

fn main() {
    let app = build_cli();
    let matches = app.clone().get_matches();
    let subcommand = matches.subcommand_name().unwrap();
    let args = matches.subcommand_matches(subcommand).unwrap();

    let result = match subcommand {
        "invalid" => commands::invalid::run(args),
        "cat" => commands::cat::run(args),
        _ => unreachable!(),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(CliError::Io(ref err))
            if err.kind() == io::ErrorKind::BrokenPipe =>
        {
            process::exit(0);
        }
        Err(CliError::Io(err)) => {
            eprintln!("IO Error: {}", err);
            process::exit(1);
        }
        Err(CliError::Pica(err)) => {
            eprintln!("Pica Error: {}", err);
            process::exit(1);
        }
    }
}
