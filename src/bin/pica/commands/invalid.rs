use crate::commands::Config;
use crate::util::{App, CliArgs, CliResult};
use clap::Arg;
use pica::Record;
use std::io::BufRead;

pub fn cli() -> App {
    App::new("invalid")
        .about("Filter out invalid records.")
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
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;
    let reader = ctx.reader(args.value_of("filename"))?;

    for line in reader.lines() {
        let line = line.unwrap();
        if Record::decode(&line).is_err() {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}
