use crate::config::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use crate::{gzip_flag, skip_invalid_flag};
use clap::Arg;
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SliceConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> App {
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
            Arg::new("gzip")
                .short('g')
                .long("gzip")
                .about("compress output with gzip"),
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

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.slice, config.global);
    let gzip_compression = gzip_flag!(args, config.slice);

    let mut reader = ReaderBuilder::new()
        .skip_invalid(false)
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
        .gzip(gzip_compression)
        .from_path_or_stdout(args.value_of("output"))?;

    // SAFETY: It's safe to call `unwrap()` because start has a default value.
    let start = match args.value_of("start").unwrap().parse::<usize>() {
        Ok(start) => start,
        Err(_) => {
            return Err(CliError::Other("invalid start option".to_string()))
        }
    };

    let end = args.value_of("end");
    let length = args.value_of("length");

    let mut range = if let Some(end) = end {
        let end = match end.parse::<usize>() {
            Ok(end) => end,
            Err(_) => {
                return Err(CliError::Other("invalid end option".to_string()))
            }
        };

        start..end
    } else if let Some(length) = length {
        let length = match length.parse::<usize>() {
            Ok(end) => end,
            Err(_) => {
                return Err(CliError::Other(
                    "invalid length option".to_string(),
                ))
            }
        };

        start..start + length
    } else {
        start..::std::usize::MAX
    };

    for (i, result) in reader.byte_records().enumerate() {
        match result {
            Ok(record) => {
                if range.contains(&i) {
                    writer.write_byte_record(&record)?;
                } else if i < range.start {
                    continue;
                } else {
                    break;
                }
            }
            Err(e) if !skip_invalid => return Err(CliError::from(e)),
            _ => {
                if length.is_some() && range.end < std::usize::MAX {
                    range.end += 1;
                }
            }
        }
    }

    writer.finish()?;
    Ok(())
}
