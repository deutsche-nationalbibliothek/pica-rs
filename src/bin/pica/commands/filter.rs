use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::{Query, Record};
use std::io::BufRead;
use std::str::FromStr;

pub fn cli() -> App {
    SubCommand::with_name("filter")
        .about("Filter records by whether the given query matches.")
        .arg(
            Arg::with_name("skip-invalid")
                .short("s")
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::with_name("invert-match")
                .short("v")
                .long("invert-match")
                .help("Filter only records that did not match."),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(
            Arg::with_name("query")
                .help("A query expression used for searching.")
                .required(true),
        )
        .arg(Arg::with_name("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;
    let reader = config.reader(args.value_of("filename"))?;

    let query_str = args.value_of("query").unwrap();
    let query = match query_str.parse::<Query>() {
        Ok(q) => q,
        _ => {
            return Err(CliError::Other(format!(
                "invalid query: {}",
                query_str
            )))
        }
    };

    let invert_match = !args.is_present("invert-match");

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(record) = Record::from_str(&line) {
            if record.matches(&query) == invert_match {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
            }
        } else if !args.is_present("skip-invalid") {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                line
            )));
        }
    }

    writer.flush()?;
    Ok(())
}
