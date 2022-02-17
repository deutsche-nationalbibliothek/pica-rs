use std::fs::File;
use std::io::{self, Write};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::util::{CliArgs, CliResult, Command};
use clap::Arg;
use pica::ReaderBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CountConfig {
    pub(crate) skip_invalid: Option<bool>,
}

pub(crate) fn cli() -> Command {
    Command::new("count")
        .about("Count records and fields.")
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
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("filename"))
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.count, config.global);

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let delimiter = if args.is_present("tsv") {
        "\t"
    } else if args.is_present("csv") {
        ","
    } else {
        " "
    };

    let mut records = 0;
    let mut fields = 0;
    let mut subfields = 0;

    for result in reader.byte_records() {
        let record = result?;

        records += 1;
        fields += record.len();
        subfields += record.iter().map(|field| field.len()).sum::<usize>();
    }

    writeln!(writer, "records{}{}", delimiter, records)?;
    writeln!(writer, "fields{}{}", delimiter, fields)?;
    writeln!(writer, "subfields{}{}", delimiter, subfields)?;

    writer.flush()?;
    Ok(())
}
