//! Filter out invalid PICA+ records.
//!
//! This command filters out invalid PICA+ records, which couldn't parsed
//! sucessfully.

use crate::cli::{App, CliArgs, CliResult};
use clap::Arg;
use pica::{Reader, ReaderBuilder, Writer, WriterBuilder};
use std::io::{Read, Write};

pub fn cli() -> App {
    App::new("cat")
        .about("Concatenate records from multiple files.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("filename").required(true))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let mut reader: Reader<Box<dyn Read>> = ReaderBuilder::new()
        .skip_invalid(args.is_present("skip-invalid"))
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Writer<Box<dyn Write>> =
        WriterBuilder::new().from_path_or_stdout(args.value_of("output"))?;

    for result in reader.byte_records() {
        writer.write_byte_record(&result?)?;
    }

    writer.flush()?;
    Ok(())
}
