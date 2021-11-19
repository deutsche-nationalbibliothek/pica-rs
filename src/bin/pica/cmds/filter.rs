use crate::common::FilterList;
use crate::config::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use crate::{gzip_flag, skip_invalid_flag};
use clap::Arg;
use pica::{
    MatcherFlags, Path, PicaWriter, ReaderBuilder, RecordMatcher, WriterBuilder,
};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct FilterConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> App {
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
            Arg::new("ignore-case")
                .short('i')
                .long("--ignore-case")
                .about("When this flag is provided, comparision operations will be search case insensitive."),
        )
        .arg(
            Arg::new("strsim-threshold")
                .long("--strsim-threshold")
                .default_value("0.75")
                .about("The minimum score for string similarity comparisons (range from 0.0..1.0).")
        )
        .arg(
            Arg::new("allow-list")
                .long("--allow-list")
                .takes_value(true)
                .multiple_occurrences(true)
        )
        .arg(
            Arg::new("deny-list")
                .long("--deny-list")
                .takes_value(true)
                .multiple_occurrences(true)
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

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.filter, config.global);
    let gzip_compression = gzip_flag!(args, config.filter);
    let ignore_case = args.is_present("ignore-case");

    let allow_list = FilterList::new(args.values_of("allow-list"))?;
    let deny_list = FilterList::new(args.values_of("deny-list"))?;

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
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
        .gzip(gzip_compression)
        .from_path_or_stdout(args.value_of("output"))?;

    let strsim_threshold = args.value_of("strsim-threshold").unwrap();
    let strsim_threshold = match strsim_threshold.parse::<f64>() {
        Err(_) => {
            return Err(CliError::Other(format!(
                "expected threshold to be a f64, got '{}'.",
                strsim_threshold,
            )));
        }
        Ok(t) if !(0.0..=1.0).contains(&t) => {
            return Err(CliError::Other(format!(
                "expected threshold between 0.0 and 1.0, got {}.",
                strsim_threshold,
            )));
        }
        Ok(threshold) => threshold,
    };

    let filter_str = if let Some(filename) = args.value_of("expr-file") {
        read_to_string(filename).unwrap()
    } else {
        args.value_of("filter").unwrap().to_owned()
    };

    let filter = match RecordMatcher::from_str(&filter_str) {
        Ok(f) => f,
        _ => {
            return Err(CliError::Other(format!(
                "invalid filter: \"{}\"",
                filter_str
            )))
        }
    };

    let mut count = 0;
    let idn_path = Path::from_str("003@.0")?;
    let flags = MatcherFlags {
        ignore_case,
        strsim_threshold,
    };

    for result in reader.byte_records() {
        let record = result?;
        let mut is_match = filter.is_match(&record, &flags);

        let idns = record.path(&idn_path);
        let idn = idns.get(0).expect("field 003@.0");

        if !allow_list.is_empty() && !allow_list.contains(idn) {
            continue;
        }

        if deny_list.contains(idn) {
            continue;
        }

        if args.is_present("invert-match") {
            is_match = !is_match;
        }

        if is_match {
            writer.write_byte_record(&record)?;
            count += 1;
        }

        if limit > 0 && count >= limit {
            break;
        }
    }

    writer.finish()?;
    Ok(())
}
