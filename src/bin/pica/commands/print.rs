use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;
use std::io::BufRead;
use std::str::FromStr;

pub fn cli() -> App {
    SubCommand::with_name("print")
        .about("Print records in human readable format.")
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

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(record) = Record::from_str(&line) {
            writer.write_all(record.pretty().as_bytes())?;
            writer.write_all(b"\n\n")?;
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
