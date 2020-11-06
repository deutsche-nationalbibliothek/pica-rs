use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;
use std::io::BufRead;
use std::str::FromStr;

pub fn cli() -> App {
    SubCommand::with_name("convert")
        .about("Serialize records to <format>.")
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
        .arg(
            Arg::with_name("format")
                .takes_value(true)
                .possible_value("json")
                .required(true),
        )
        .arg(Arg::with_name("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let ctx = Config::new();
    let mut writer = ctx.writer(args.value_of("output"))?;
    let reader = ctx.reader(args.value_of("filename"))?;
    let format = args.value_of("format").unwrap();
    let skip_invalid = args.is_present("skip-invalid");

    writer.write_all(b"[")?;

    for (count, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        if let Ok(record) = Record::from_str(&line) {
            let serialized = match format {
                "json" => {
                    if count > 0 {
                        writer.write_all(b",")?;
                    }
                    serde_json::to_string(&record).unwrap()
                }
                _ => unreachable!(),
            };

            writer.write_all(serialized.as_bytes())?;
        } else if !skip_invalid {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                line
            )));
        }
    }

    writer.write_all(b"]")?;
    writer.flush()?;

    Ok(())
}
