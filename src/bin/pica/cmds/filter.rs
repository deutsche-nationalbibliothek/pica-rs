use crate::config::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{Filter, PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilterConfig {
    pub skip_invalid: Option<bool>,
    pub gzip: Option<bool>,
}

pub fn cli() -> App {
    App::new("filter")
        .about("Filter records by whether the given query matches.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("expr-file")
                .short('f')
                .long("file")
                .value_name("file")
                .about("Take filter expressions from file.")
                .takes_value(true),
        )
        .arg(
            Arg::new("invert-match")
                .short('v')
                .long("invert-match")
                .about("Filter only records that did not match."),
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("--limit")
                .value_name("n")
                .about("Limit the result to first <n> records."),
        )
        .arg(
            Arg::new("gzip")
                .short('g')
                .long("gzip")
                .about("compress output with gzip"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(
            Arg::new("filter")
                .about("A filter expression used for searching.")
                .required(true),
        )
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = match args.is_present("skip-invalid") {
        false => {
            if let Some(ref filter_config) = config.filter {
                filter_config.skip_invalid.unwrap_or_default()
            } else if let Some(ref global_config) = config.global {
                global_config.skip_invalid.unwrap_or_default()
            } else {
                false
            }
        }
        _ => true,
    };

    let gzip_compression = match args.is_present("gzip") {
        false => {
            if let Some(ref filter_config) = config.filter {
                filter_config.gzip.unwrap_or_default()
            } else {
                false
            }
        }
        _ => true,
    };

    let limit = match args.value_of("limit").unwrap_or("0").parse::<usize>() {
        Ok(limit) => limit,
        Err(_) => {
            return Err(CliError::Other(
                "Invalid limit value, expected unsigned integer.".to_string(),
            ));
        }
    };

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .limit(limit)
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
        .gzip(gzip_compression)
        .from_path_or_stdout(args.value_of("output"))?;

    let filter_str = if let Some(filename) = args.value_of("expr-file") {
        read_to_string(filename).unwrap()
    } else {
        args.value_of("filter").unwrap().to_owned()
    };

    let filter = match Filter::decode(&filter_str) {
        Ok(f) => f,
        _ => {
            return Err(CliError::Other(format!(
                "invalid filter: \"{}\"",
                filter_str
            )))
        }
    };

    for result in reader.byte_records() {
        let record = result?;
        let mut is_match = filter.matches(&record);

        if args.is_present("invert-match") {
            is_match = !is_match;
        }

        if is_match {
            writer.write_byte_record(&record)?;
        }
    }

    writer.finish()?;
    Ok(())
}
