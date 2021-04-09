use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{Error, ParsePicaError, ReaderBuilder};
use std::fs::File;
use std::io::{self, BufWriter, Write};

pub fn cli() -> App {
    App::new("invalid")
        .about("Filter out invalid records.")
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
    let mut reader = ReaderBuildero::new()
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
