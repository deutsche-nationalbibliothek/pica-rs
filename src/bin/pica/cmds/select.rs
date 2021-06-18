use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{Outcome, ReaderBuilder, Selectors};
use std::fs::File;
use std::io::{self, Write};

pub fn cli() -> App {
    App::new("select")
        .about("Select fields from a record.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("tsv")
                .short('t')
                .long("tsv")
                .about("use tabs as field delimiter"),
        )
        .arg(
            Arg::new("header")
                .short('H')
                .long("--header")
                .value_name("header")
                .about("Comma-separated list of column names."),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("selectors").required(true))
        .arg(Arg::new("filename"))
}

fn writer(filename: Option<&str>) -> CliResult<Box<dyn Write>> {
    Ok(match filename {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    })
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let mut reader = ReaderBuilder::new()
        .skip_invalid(args.is_present("skip-invalid"))
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer = csv::WriterBuilder::new()
        .delimiter(if args.is_present("tsv") { b'\t' } else { b',' })
        .from_writer(writer(args.value_of("output"))?);

    let selectors_str = args.value_of("selectors").unwrap();
    let selectors = match Selectors::decode(selectors_str) {
        Ok(val) => val,
        _ => {
            return Err(CliError::Other(format!(
                "invalid select list: {}",
                selectors_str
            )))
        }
    };

    if let Some(header) = args.value_of("header") {
        writer.write_record(header.split(',').map(|s| s.trim()))?;
    }

    for result in reader.records() {
        let record = result?;
        let outcome = selectors
            .iter()
            .map(|selector| record.select(selector))
            .fold(Outcome::default(), |acc, x| acc * x);

        for row in outcome.iter() {
            if !row.iter().all(|col| col.is_empty()) {
                writer.write_record(row)?;
            }
        }
    }

    writer.flush()?;
    Ok(())
}
