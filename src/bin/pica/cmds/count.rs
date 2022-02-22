use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};

use clap::Arg;
use pica::{Reader, ReaderBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::util::{CliArgs, CliResult, Command};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CountConfig {
    pub(crate) skip_invalid: Option<bool>,
}

pub(crate) fn cli() -> Command {
    Command::new("count")
        .about("Count records, fields and subfields..")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::new("tsv")
                .long("tsv")
                .help("use tabs as delimiter")
                .conflicts_with("csv"),
        )
        .arg(
            Arg::new("csv")
                .long("csv")
                .help("use tabs as delimiter")
                .conflicts_with("tsv"),
        )
        .arg(
            Arg::new("no-header")
                .long("--no-header")
                .help("Do not write header row.")
        )
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

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.count, config.global);

    let mut writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut records = 0;
    let mut fields = 0;
    let mut subfields = 0;

    let filenames = args
        .values_of_t::<OsString>("filenames")
        .unwrap_or_else(|_| vec![OsString::from("-")]);

    for filename in filenames {
        let builder = ReaderBuilder::new().skip_invalid(skip_invalid);
        let mut reader: Reader<Box<dyn Read>> = match filename.to_str() {
            Some("-") => builder.from_reader(Box::new(io::stdin())),
            _ => builder.from_path(filename)?,
        };

        for result in reader.byte_records() {
            let record = result?;

            records += 1;
            fields += record.len();
            subfields += record.iter().map(|field| field.len()).sum::<usize>();
        }
    }

    if args.is_present("csv") {
        if !args.is_present("no-header") {
            writeln!(writer, "records,fields,subfields")?;
        }
        writeln!(writer, "{},{},{}", records, fields, subfields)?;
    } else if args.is_present("tsv") {
        if !args.is_present("no-header") {
            writeln!(writer, "records\tfields\tsubfields")?;
        }
        writeln!(writer, "{}\t{}\t{}", records, fields, subfields)?;
    } else {
        writeln!(writer, "records: {}", records)?;
        writeln!(writer, "fields: {}", fields)?;
        writeln!(writer, "subfields: {}", subfields)?;
    }

    writer.flush()?;
    Ok(())
}
