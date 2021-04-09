use crate::util::{App, CliArgs, CliResult};
use clap::Arg;
use pica::{ReaderBuilder, Writer, WriterBuilder};
use std::io::Write;

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
    let mut reader = ReaderBuilder::new()
        .skip_invalid(args.is_present("skip-invalid"))
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Writer<Box<dyn Write>> =
        WriterBuilder::new().from_path_or_stdout(args.value_of("output"))?;

    for result in reader.records() {
        writer.write_all(&format!("{}\n\n", result?).as_bytes())?;
    }

    writer.flush()?;
    Ok(())
}
