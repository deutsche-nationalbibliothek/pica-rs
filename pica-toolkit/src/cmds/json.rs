use std::ffi::OsString;
use std::io::{self, Read, Write};

use clap::Arg;
use pica_api::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
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
        .arg(
            Arg::new("filenames")
                .help(
                    "Read one or more files in normalized PICA+ format. If the file \
                    ends with .gz the content is automatically decompressed. With no \
                    <filenames>, or when filename is -, read from standard input (stdin).")
                .value_name("filenames")
                .multiple_values(true)
        )
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.json, config.global);

    let mut writer: Box<dyn PicaWriter> =
        WriterBuilder::new().from_path_or_stdout(args.value_of("output"))?;

    writer.write_all(b"[")?;

    let filenames = args
        .values_of_t::<OsString>("filenames")
        .unwrap_or_else(|_| vec![OsString::from("-")]);

    for filename in filenames {
        let builder = ReaderBuilder::new().skip_invalid(skip_invalid);
        let mut reader: Reader<Box<dyn Read>> = match filename.to_str() {
            Some("-") => builder.from_reader(Box::new(io::stdin())),
            _ => builder.from_path(filename)?,
        };
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
    }

    writer.write_all(b"]")?;
    writer.flush()?;

    Ok(())
}
