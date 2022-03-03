use std::io::Write;

use clap::Arg;
use pica::{PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::{CliArgs, CliResult, Command};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct JsonConfig {
    pub(crate) skip_invalid: Option<bool>,
}

pub(crate) fn cli() -> Command {
    Command::new("json")
        .about("Serialize records to JSON.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::new("translit")
                .long("--translit")
                .value_name("translit")
                .possible_values(["nfd", "nfkd", "nfc", "nfkc"])
                .help("If present, transliterate output into the selected normalform.")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("filename"))
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.json, config.global);

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut writer: Box<dyn PicaWriter> =
        WriterBuilder::new().from_path_or_stdout(args.value_of("output"))?;

    writer.write_all(b"[")?;

    for (count, result) in reader.records().enumerate() {
        let record = result?;

        if count > 0 {
            writer.write_all(b",")?;
        }

        let j = translit_maybe(
            &serde_json::to_string(&record).unwrap(),
            args.value_of("translit"),
        );
        writer.write_all(j.as_bytes())?;
    }

    writer.write_all(b"]")?;
    writer.flush()?;

    Ok(())
}
