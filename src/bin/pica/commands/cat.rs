use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;
use std::io::BufRead;
use std::str::FromStr;

pub fn cli() -> App {
    SubCommand::with_name("cat")
        .about("Concatenate records from multiple files.")
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
        .arg(Arg::with_name("filenames").multiple(true).required(true))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;
    let skip_invalid = args.is_present("skip-invalid");

    for filename in args.values_of("filenames").unwrap() {
        let reader = config.reader(Some(filename))?;

        for line in reader.lines() {
            let line = line.unwrap();
            if let Ok(_record) = Record::from_str(&line) {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
            } else if !skip_invalid {
                return Err(CliError::Other(format!(
                    "could not read record: {}",
                    line
                )));
            }
        }
    }

    writer.flush()?;
    Ok(())
}
