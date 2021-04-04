//! Filter out invalid PICA+ records.
//!
//! This command filters out invalid PICA+ records, which couldn't parsed
//! sucessfully.

use crate::cli::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{Error, ParsePicaError, Reader, ReaderBuilder};
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};

pub fn cli() -> App {
    App::new("invalid")
        .about("Filter out invalid PICA+ records.")
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let mut reader: Reader<Box<dyn Read>> = ReaderBuilder::new()
        .skip_invalid(false)
        .from_path_or_stdin(args.value_of("filename"))?;

    let writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut writer = BufWriter::new(writer);

    for result in reader.records() {
        match result {
            Err(Error::InvalidRecord(ParsePicaError { data, .. })) => {
                writer.write_all(&data)?;
            }
            Err(e) => return Err(CliError::from(e)),
            _ => continue,
        }
    }

    writer.flush()?;
    Ok(())
}
