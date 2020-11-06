#[macro_use]
extern crate clap;

mod commands;
mod util;

use clap::{App, AppSettings};
use std::process;
use util::CliError;

fn main() {
    let m = App::new("pica")
        .about("Tools to work with bibliographic records encoded in Pica+")
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .author(crate_authors!())
        .subcommands(commands::subcmds())
        .get_matches();

    let name = m.subcommand_name().unwrap();
    let args = m.subcommand_matches(name).unwrap();

    let result = match name {
        "cat" => commands::cat::run(args),
        "filter" => commands::filter::run(args),
        "print" => commands::print::run(args),
        "sample" => commands::sample::run(args),
        _ => unreachable!(),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(CliError::Io(err)) => {
            eprintln!("IO Error: {}", err);
            process::exit(1);
        }
        Err(CliError::Other(err)) => {
            eprintln!("error: {}", err);
            process::exit(1);
        }
    }
}
