use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;
use std::io::BufRead;

pub fn cli() -> App {
    SubCommand::with_name("json")
        .about("Serialize records to JSON.")
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
        .arg(Arg::with_name("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let ctx = Config::new();
    let mut writer = ctx.writer(args.value_of("output"))?;
    let reader = ctx.reader(args.value_of("filename"))?;
    let skip_invalid = args.is_present("skip-invalid");

    writer.write_all(b"[")?;

    for (count, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        if let Ok(record) = Record::decode(&line) {
            let j = serde_json::to_string(&record).unwrap();
            if count > 0 {
                writer.write_all(b",")?;
            }
            writer.write_all(j.as_bytes())?;
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
