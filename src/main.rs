#[macro_use]
extern crate clap;
extern crate csv;
extern crate regex;

mod cli;
mod cmds;
mod util;

use std::io;
use std::process;
use util::CliError;

fn main() {
    let mut app = cli::build_cli();
    let m = app.clone().get_matches();
    let name = m.subcommand_name().unwrap();
    let args = m.subcommand_matches(name).unwrap();

    let result = match name {
        "cat" => cmds::cat::run(args),
        "completion" => cmds::completion::run(args, &mut app),
        "filter" => cmds::filter::run(args),
        "frequency" => cmds::frequency::run(args),
        "invalid" => cmds::invalid::run(args),
        "json" => cmds::json::run(args),
        "partition" => cmds::partition::run(args),
        "print" => cmds::print::run(args),
        "sample" => cmds::sample::run(args),
        "select" => cmds::select::run(args),
        "split" => cmds::split::run(args),
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
        Err(CliError::Csv(err)) => {
            eprintln!("CSV Error: {}", err);
            process::exit(1);
        }
        Err(CliError::Other(err)) => {
            eprintln!("error: {}", err);
            process::exit(1);
        }
    }
}
