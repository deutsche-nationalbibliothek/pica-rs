use crate::cmds::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::Filter;
use pica::Record;
use std::io::BufRead;

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
            Arg::new("invert-match")
                .short('v')
                .long("invert-match")
                .about("Filter only records that did not match."),
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

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;
    let reader = config.reader(args.value_of("filename"))?;
    let skip_invalid = args.is_present("skip-invalid");

    let filter_str = args.value_of("filter").unwrap();
    let filter = match Filter::decode(filter_str) {
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
        if let Ok(record) = Record::decode(&line) {
            if filter.matches(&record) == invert_match {
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
