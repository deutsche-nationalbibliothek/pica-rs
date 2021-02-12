use crate::cmds::Config;
use crate::util::{App, CliArgs, CliResult};
use bstr::io::BufReadExt;
use clap::Arg;
use pica::new::Record;

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
    let ctx = Config::new();
    let mut writer = ctx.writer(args.value_of("output"))?;
    let reader = ctx.reader(args.value_of("filename"))?;

    for result in reader.byte_lines() {
        let line = result.unwrap();
        if Record::from_bytes(&line).is_err() {
            writer.write_all(&line)?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}
