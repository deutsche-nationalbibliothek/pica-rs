use std::io::ErrorKind;
use std::process::ExitCode;

use clap::Parser;
use cli::{Args, Command};
use error::{CliError, CliResult};

mod cli;
mod commands;
mod error;
mod progress;

fn run(args: Args) -> CliResult {
    match args.cmd {
        Command::Invalid(cmd) => cmd.execute(),
    }
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(code) => code,
        Err(CliError::IO(err))
            if err.kind() == ErrorKind::BrokenPipe =>
        {
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}
