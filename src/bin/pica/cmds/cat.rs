use crate::cmds::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use bstr::io::BufReadExt;
use clap::Arg;
use pica::new::Record;

pub fn cli() -> App {
    App::new("cat")
        .about("Concatenate records from multiple files.")
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
        .arg(Arg::new("filenames").multiple(true).required(true))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;
    let skip_invalid = args.is_present("skip-invalid");

    for filename in args.values_of("filenames").unwrap() {
        let reader = config.reader(Some(filename))?;

        for result in reader.byte_lines() {
            let line = result?;

            if Record::from_bytes(&line).is_ok() {
                writer.write_all(&line)?;
                writer.write_all(b"\n")?;
            } else if !skip_invalid {
                return Err(CliError::Other(format!(
                    "could not read record: {}",
                    String::from_utf8(line).unwrap()
                )));
            }
        }
    }

    writer.flush()?;
    Ok(())
}
