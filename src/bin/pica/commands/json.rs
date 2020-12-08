use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::Record;
use std::io::BufRead;

pub fn cli() -> App {
    App::new("json")
        .about("Serialize records to JSON.")
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
