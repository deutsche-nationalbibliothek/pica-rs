use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{Filter, ReaderBuilder, Writer, WriterBuilder};
use std::io::Write;

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
            Arg::new("limit")
                .short('l')
                .long("--limit")
                .value_name("n")
                .about("Limit the result to first <n> records."),
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
    let limit = match args.value_of("limit").unwrap_or("0").parse::<usize>() {
        Ok(limit) => limit,
        Err(_) => {
            return Err(CliError::Other(
                "Invalid limit value, expected unsigned integer.".to_string(),
            ));
        }
    };

    let mut reader = ReaderBuilder::new()
        .skip_invalid(args.is_present("skip-invalid"))
        .limit(limit)
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Writer<Box<dyn Write>> =
        WriterBuilder::new().from_path_or_stdout(args.value_of("output"))?;

    let filter_str = args.value_of("filter").unwrap();
    let filter = match Filter::decode(filter_str) {
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

    writer.flush()?;
    Ok(())
}
