use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;
use std::str::FromStr;

pub fn cli() -> App {
    SubCommand::with_name("print")
        .about("Print records in human readable format.")
        .arg(
            Arg::with_name("skip-invalid")
                .short("s")
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::with_name("FILENAME"))
}

use std::{
    boxed::Box,
    fs::File,
    io,
    io::{BufRead, BufReader, Write},
};

pub fn run(args: &CliArgs) -> CliResult<()> {
    let mut writer: Box<dyn Write> = match args.value_of("output") {
        None => Box::new(io::stdout()),
        Some(filename) => Box::new(File::create(filename)?),
    };

    let reader: Box<dyn BufRead> = match args.value_of("FILENAME") {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(File::open(filename)?)),
    };

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(record) = Record::from_str(&line) {
            writer.write(record.to_string().as_bytes())?;
            writer.write(b"\n").unwrap();
        } else if !args.is_present("skip-invalid") {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                line
            )));
        }
    }

    Ok(())
}
