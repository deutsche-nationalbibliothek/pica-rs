use crate::commands::Config;
use crate::util::{App, CliArgs, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;
use std::io::BufRead;

pub fn cli() -> App {
    SubCommand::with_name("invalid")
        .about("Filter out invalid records.")
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::with_name("filenames").multiple(true).required(true))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;

    for filename in args.values_of("filenames").unwrap() {
        let reader = config.reader(Some(filename))?;

        for line in reader.lines() {
            let line = line.unwrap();
            if Record::decode(&line).is_err() {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
            }
        }
    }

    writer.flush()?;
    Ok(())
}
