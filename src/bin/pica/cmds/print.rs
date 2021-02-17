use crate::cmds::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::legacy::Record;
use std::io::BufRead;

pub fn cli() -> App {
    App::new("print")
        .about("Print records in human readable format.")
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
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let ctx = Config::new();
    let mut writer = ctx.writer(args.value_of("output"))?;
    let reader = ctx.reader(args.value_of("filename"))?;
    let skip_invalid = args.is_present("skip-invalid");

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(record) = Record::decode(&line) {
            writer.write_all(record.pretty().as_bytes())?;
            writer.write_all(b"\n\n")?;
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
