use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};

use clap::Arg;
use pica::{Error, ParsePicaError, Reader, ReaderBuilder};

use crate::util::{CliArgs, CliError, CliResult, Command};

pub(crate) fn cli() -> Command {
    Command::new("invalid")
        .about("Filter out invalid records.")
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(
            Arg::new("filenames")
                .help(
                    "Read one or more files in normalized PICA+ format. If the file \
                    ends with .gz the content is automatically decompressed. With no \
                    <filenames>, or when filename is -, read from standard input (stdin).")
                .value_name("filenames")
                .multiple_values(true)
        )
}

pub(crate) fn run(args: &CliArgs) -> CliResult<()> {
    let writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut writer = BufWriter::new(writer);

    let filenames = args
        .values_of_t::<OsString>("filenames")
        .unwrap_or_else(|_| vec![OsString::from("-")]);

    for filename in filenames {
        let builder = ReaderBuilder::new().skip_invalid(false);
        let mut reader: Reader<Box<dyn Read>> = match filename.to_str() {
            Some("-") => builder.from_reader(Box::new(io::stdin())),
            _ => builder.from_path(filename)?,
        };

        for result in reader.records() {
            match result {
                Err(Error::InvalidRecord(ParsePicaError { data, .. })) => {
                    writer.write_all(&data)?;
                }
                Err(e) => return Err(CliError::from(e)), // no-coverage
                _ => continue,
            }
        }
    }

    writer.flush()?;
    Ok(())
}
