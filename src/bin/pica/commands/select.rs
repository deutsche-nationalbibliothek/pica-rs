use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::parse_path_list;
use pica::Record;
use std::io::BufRead;

pub fn cli() -> App {
    SubCommand::with_name("select")
        .about("Select fields from a record.")
        .arg(
            Arg::with_name("skip-invalid")
                .short("s")
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::with_name("fields").required(true))
        .arg(Arg::with_name("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let ctx = Config::new();
    let writer = ctx.writer(args.value_of("output"))?;
    let mut writer = csv::Writer::from_writer(writer);
    let reader = ctx.reader(args.value_of("filename"))?;
    let skip_invalid = args.is_present("skip-invalid");

    let fields_str = args.value_of("fields").unwrap();
    let paths = match parse_path_list(fields_str) {
        Ok((_, paths)) => paths,
        _ => {
            return Err(CliError::Other(format!(
                "invalid field list: {}",
                fields_str
            )))
        }
    };

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(record) = Record::decode(&line) {
            let mut rows = Vec::<Vec<&str>>::new();

            for path in &paths {
                let mut values = record.path(&path);
                if values.is_empty() {
                    values = vec![""];
                }

                if rows.is_empty() {
                    for value in values {
                        rows.push(vec![value]);
                    }
                } else {
                    let mut temp = Vec::<Vec<&str>>::new();

                    for row in &mut rows {
                        for value in &values {
                            let mut new_row = row.clone();
                            new_row.push(value);
                            temp.push(new_row);
                        }
                    }

                    rows = temp;
                }
            }

            for row in rows {
                writer.write_record(row)?;
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
