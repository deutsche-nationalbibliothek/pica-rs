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
pub(crate) mod prelude;
mod progress;
mod translit;
mod utils;

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
        Command::Convert(cmd) => cmd.execute(&config),
        Command::Count(cmd) => cmd.execute(&config),
        Command::Explode(cmd) => cmd.execute(&config),
        Command::Filter(cmd) => cmd.execute(&config),
        Command::Frequency(cmd) => cmd.execute(&config),
        Command::Hash(cmd) => cmd.execute(&config),
        Command::Invalid(cmd) => cmd.execute(&config),
        Command::Partition(cmd) => cmd.execute(&config),
        Command::Print(cmd) => cmd.execute(&config),
        Command::Sample(cmd) => cmd.execute(&config),
        Command::Slice(cmd) => cmd.execute(&config),
        Command::Split(cmd) => cmd.execute(&config),
        Command::Select(cmd) => cmd.execute(&config),
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
