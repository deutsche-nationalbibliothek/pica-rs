use crate::config::Config;
use crate::skip_invalid_flag;
use crate::util::{App, CliArgs, CliResult};
use clap::Arg;
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct JsonConfig {
    pub skip_invalid: Option<bool>,
}

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

pub fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.json, config.global);

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Box<dyn PicaWriter> =
        WriterBuilder::new().from_path_or_stdout(args.value_of("output"))?;

    writer.write_all(b"[")?;

    for (count, result) in reader.records().enumerate() {
        let record = result?;

        let j = serde_json::to_string(&record).unwrap();
        if count > 0 {
            writer.write_all(b",")?;
        }
        writer.write_all(j.as_bytes())?;
    }

    writer.write_all(b"]")?;
    writer.flush()?;

    Ok(())
}
