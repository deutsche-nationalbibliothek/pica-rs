use crate::cmds::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use bstr::io::BufReadExt;
use clap::Arg;
use pica::Record;

pub fn cli() -> App {
    App::new("slice")
        .about("Return records within a range (half-open interval).")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("start")
                .long("start")
                .about("The lower bound of the range (inclusive).")
                .default_value("0"),
        )
        .arg(
            Arg::new("end")
                .long("end")
                .about("The upper bound of the range (exclusive).")
                .takes_value(true),
        )
        .arg(
            Arg::new("length")
                .long("length")
                .about("The length of the slice.")
                .conflicts_with("end")
                .takes_value(true),
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
    let config = Config::new();
    let mut writer = config.writer(args.value_of("output"))?;
    let reader = config.reader(args.value_of("filename"))?;
    let skip_invalid = args.is_present("skip-invalid");

    let start = args.value_of("start").unwrap().parse::<usize>().unwrap();
    let end = args.value_of("end");
    let length = args.value_of("length");

    let mut range = if let Some(end) = end {
        start..end.parse::<usize>().unwrap()
    } else if let Some(length) = length {
        let length = length.parse::<usize>().unwrap();
        start..start + length
    } else {
        start..::std::usize::MAX
    };

    for (i, result) in reader.byte_lines().enumerate() {
        let line = result?;

        if let Ok(_record) = Record::from_bytes(&line) {
            if range.contains(&i) {
                writer.write_all(&line)?;
                writer.write_all(b"\n")?;
            } else if i < range.start {
                continue;
            } else {
                break;
            }
        } else if !skip_invalid {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                String::from_utf8(line).unwrap()
            )));
        } else if length.is_some() {
            range.end += 1;
        }
    }

    writer.flush()?;
    Ok(())
}
