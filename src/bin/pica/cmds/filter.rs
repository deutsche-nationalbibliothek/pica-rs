use crate::common::FilterList;
use crate::config::Config;
use crate::util::{CliArgs, CliError, CliResult, Command};
use crate::{gzip_flag, skip_invalid_flag};
use clap::Arg;
use pica::matcher::{MatcherFlags, RecordMatcher, TagMatcher};
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct FilterConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> Command {
    Command::new("filter")
        .about("Filter records by whether the given query matches.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::new("expr-file")
                .short('f')
                .long("file")
                .value_name("file")
                .help("Take filter expressions from file.")
                .takes_value(true),
        )
        .arg(
            Arg::new("invert-match")
                .short('v')
                .long("invert-match")
                .help("Filter only records that did not match."),
        )
        .arg(
            Arg::new("ignore-case")
                .short('i')
                .long("--ignore-case")
                .help("When this flag is provided, comparision operations will be search case insensitive."),
        )
        .arg(
            Arg::new("strsim-threshold")
                .long("--strsim-threshold")
                .default_value("0.75")
                .help("The minimum score for string similarity comparisons (range from 0.0..1.0).")
        )
        .arg(
            Arg::new("reduce")
                .long("reduce")
                .help("Reduce the record to the following fields.")
                .takes_value(true),
        )
        .arg(
            Arg::new("and")
                .long("and")
                .takes_value(true)
                .multiple_occurrences(true),
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
                .help("Limit the result to first <n> records."),
        )
        .arg(
            Arg::new("gzip")
                .short('g')
                .long("gzip")
                .help("compress output with gzip"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(
            Arg::new("filter")
                .help("A filter expression used for searching.")
                .required(true),
        )
        .arg(Arg::new("filename"))
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.filter, config.global);
    let gzip_compression = gzip_flag!(args, config.filter);
    let ignore_case = args.is_present("ignore-case");

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

    let mut reducers = vec![];
    if let Some(reduce_expr) = args.value_of("reduce") {
        reducers = reduce_expr
            .split(',')
            .map(TagMatcher::new)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CliError::Other("invalid reduce value".to_string()))?;
    }

    let filter_str = if let Some(filename) = args.value_of("expr-file") {
        read_to_string(filename).unwrap()
    } else {
        args.value_of("filter").unwrap().to_owned()
    };

    let mut filter = match RecordMatcher::new(&filter_str) {
        Ok(f) => f,
        _ => {
            return Err(CliError::Other(format!(
                "invalid filter: \"{}\"",
                filter_str
            )))
        }
    };

    if args.is_present("and") {
        let predicates = args
            .values_of("and")
            .unwrap()
            .map(RecordMatcher::new)
            .collect::<Result<Vec<_>, _>>()?;

        for expr in predicates.into_iter() {
            filter = RecordMatcher::Group(Box::new(filter))
                & RecordMatcher::Group(Box::new(expr));
        }
    }

    if let Some(allow_lists) = args.values_of("allow-list") {
        let allow_list =
            FilterList::new(allow_lists.collect::<Vec<&str>>(), false)?;
        let matcher = RecordMatcher::from(allow_list);

        filter = matcher & filter;
    }

    if let Some(deny_lists) = args.values_of("deny-list") {
        let deny_list =
            FilterList::new(deny_lists.collect::<Vec<&str>>(), true)?;
        let matcher = RecordMatcher::from(deny_list);

        filter = matcher & filter;
    }

    let mut count = 0;
    let flags = MatcherFlags {
        ignore_case,
        strsim_threshold,
    };

    for result in reader.byte_records() {
        let mut record = result?;
        let mut is_match = filter.is_match(&record, &flags);

        if args.is_present("invert-match") {
            is_match = !is_match;
        }

        if is_match {
            if !reducers.is_empty() {
                record.reduce(&reducers);
            }

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
