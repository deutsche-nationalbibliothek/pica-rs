use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::Record;
use pica::Selectors;
use std::io::BufRead;

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
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("fields").required(true))
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let ctx = Config::new();
    let writer = ctx.writer(args.value_of("output"))?;
    let mut writer = csv::Writer::from_writer(writer);
    let reader = ctx.reader(args.value_of("filename"))?;
    let skip_invalid = args.is_present("skip-invalid");

    let selectors_str = args.value_of("fields").unwrap();
    let selectors = match Selectors::parse(&selectors_str) {
        Ok(val) => val,
        _ => {
            return Err(CliError::Other(format!(
                "invalid select list: {}",
                selectors_str
            )))
        }
    };

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(record) = Record::decode(&line) {
            let rows = record.select(&selectors);
            for row in rows {
                if row.iter().all(|r| !r.is_empty()) {
                    writer.write_record(row)?;
                }
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
