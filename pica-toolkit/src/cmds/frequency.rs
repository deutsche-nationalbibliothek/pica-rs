use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::str::FromStr;

use bstr::BString;
use clap::Arg;
use pica::Path;
use pica_api::{Reader, ReaderBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::{CliArgs, CliError, CliResult, Command};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct FrequencyConfig {
    pub(crate) skip_invalid: Option<bool>,
}

pub(crate) fn cli() -> Command {
    Command::new("frequency")
        .about("Compute a frequency table of a subfield.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::new("reverse")
                .short('r')
                .long("reverse")
                .help("Sort results in reverse order."),
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("--limit")
                .value_name("n")
                .help("Limit the result to the <n> most common items."),
        )
        .arg(
            Arg::new("threshold")
                .short('t')
                .long("--threshold")
                .value_name("t")
                .help("Ignore rows with a frequency ≤ <t>."),
        )
        .arg(
            Arg::new("header")
                .short('H')
                .long("--header")
                .value_name("header")
                .help("Comma-separated list of column names."),
        )
        .arg(
            Arg::new("translit")
                .long("--translit")
                .value_name("translit")
                .possible_values(["nfd", "nfkd", "nfc", "nfkc"])
                .help("If present, transliterate output into the selected normalform.")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("path").required(true))
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
    let skip_invalid =
        skip_invalid_flag!(args, config.frequency, config.global);

    let limit = match args.value_of("limit").unwrap_or("0").parse::<usize>() {
        Ok(limit) => limit,
        Err(_) => {
            return Err(CliError::Other(
                "Invalid limit value, expected unsigned integer.".to_string(),
            ));
        }
    };

    let threshold =
        match args.value_of("threshold").unwrap_or("0").parse::<u64>() {
            Ok(threshold) => threshold,
            Err(_) => {
                return Err(CliError::Other(
                    "Invalid threshold value, expected unsigned integer."
                        .to_string(),
                ));
            }
        };

    let mut ftable: HashMap<BString, u64> = HashMap::new();
    let path = Path::from_str(args.value_of("path").unwrap())?;

    let writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut writer = csv::WriterBuilder::new().from_writer(writer);

    let filenames = args
        .values_of_t::<OsString>("filenames")
        .unwrap_or_else(|_| vec![OsString::from("-")]);

    for filename in filenames {
        let builder = ReaderBuilder::new().skip_invalid(skip_invalid);
        let mut reader: Reader<Box<dyn Read>> = match filename.to_str() {
            Some("-") => builder.from_reader(Box::new(io::stdin())),
            _ => builder.from_path(filename)?,
        };

        for result in reader.records() {
            let record = result?;

            for value in record.path(&path) {
                *ftable.entry(value.to_owned()).or_insert(0) += 1;
            }
        }
    }

    if let Some(header) = args.value_of("header") {
        writer.write_record(header.split(',').map(|s| s.trim()))?;
    }

    let mut ftable_sorted: Vec<(&BString, &u64)> = ftable.iter().collect();
    if args.is_present("reverse") {
        ftable_sorted.sort_by(|a, b| a.1.cmp(b.1));
    } else {
        ftable_sorted.sort_by(|a, b| b.1.cmp(a.1));
    }

    for (i, (value, frequency)) in ftable_sorted.iter().enumerate() {
        if limit > 0 && i >= limit {
            break;
        }

        if **frequency <= threshold {
            break;
        }

        let value =
            translit_maybe(&value.to_string(), args.value_of("translit"));
        writer.write_record(&[value, frequency.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}
