use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::{Filter, Record};
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
            Arg::with_name("filter")
                .help("A filter expression used for searching.")
                .required(true),
        )
        .arg(Arg::with_name("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;
    let reader = config.reader(args.value_of("filename"))?;
    let skip_invalid = args.is_present("skip-invalid");

    let filter_str = args.value_of("filter").unwrap();
    let filter = match filter_str.parse::<Filter>() {
        Ok(f) => f,
        _ => {
            return Err(CliError::Other(format!(
                "invalid filter: {}",
                filter_str
            )))
        }
    };

    let invert_match = !args.is_present("invert-match");

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(record) = Record::from_str(&line) {
            if record.matches(&filter) == invert_match {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
            }
        } else if !skip_invalid {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                line
            )));
        }
    }

    writer.flush()?;
    Ok(())
}
