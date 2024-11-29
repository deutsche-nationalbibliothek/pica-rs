use std::io::ErrorKind;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use cli::{Args, Command};
use config::Config;
use error::{CliError, CliResult};

mod cli;
mod commands;
mod config;
mod error;
mod progress;
mod unicode;

fn run() -> CliResult {
    let args = Args::parse();

    #[allow(unused_mut)]
    let mut config = if let Some(ref path) = args.config {
        Config::from_path(path).unwrap_or(Config::new(path))
    } else {
        Config::discover()?
    };

    match args.cmd {
        Command::Completions(cmd) => cmd.execute(&mut Args::command()),
        Command::Concat(cmd) => cmd.execute(&config),
        #[cfg(feature = "unstable")]
        Command::Config(cmd) => cmd.execute(&mut config),
        Command::Invalid(cmd) => cmd.execute(),
    }
}

fn main() -> ExitCode {
    match run() {
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
