use crate::config::Config;
use crate::skip_invalid_flag;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PrintConfig {
    pub skip_invalid: Option<bool>,
}

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
            Arg::new("limit")
                .short('l')
                .long("--limit")
                .value_name("n")
                .about("Limit the result to first <n> records."),
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

pub fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.print, config.global);

    let limit = match args.value_of("limit").unwrap_or("0").parse::<usize>() {
        Ok(limit) => limit,
        Err(_) => {
            return Err(CliError::Other(
                "Invalid limit value, expected unsigned integer.".to_string(),
            ));
        }
    };

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .limit(limit)
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Box<dyn PicaWriter> =
        WriterBuilder::new().from_path_or_stdout(args.value_of("output"))?;

    for result in reader.records() {
        writer.write_all(format!("{}\n\n", result?).as_bytes())?;
    }

    writer.flush()?;
    Ok(())
}
