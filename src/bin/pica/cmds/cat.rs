use std::ffi::OsString;
use std::io::{self, Read};

use crate::config::Config;
use crate::util::{CliArgs, CliResult, Command};
use crate::{gzip_flag, skip_invalid_flag};
use clap::Arg;
use pica::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CatConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> Command {
    Command::new("cat")
        .about("Concatenate records from multiple files.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::new("gzip")
                .short('g')
                .long("gzip")
                .help("compress output with gzip")
                .requires("output"),
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
                    "Concatenate all records from <filenames> into \
                    one stream. With no <filenames>, or when a filename \
                    is -, read from standard input (stdint).",
                )
                .multiple_values(true),
        )
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.cat, config.global);
    let gzip_compression = gzip_flag!(args, config.cat);

    let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
        .gzip(gzip_compression)
        .from_path_or_stdout(args.value_of("output"))?;

    let filenames = args
        .values_of_t::<OsString>("filenames")
        .unwrap_or_else(|_| vec![OsString::from("-")]);

    for filename in filenames {
        let builder = ReaderBuilder::new().skip_invalid(skip_invalid);
        let mut reader: Reader<Box<dyn Read>> = match filename.to_str() {
            Some("-") => builder.from_reader(Box::new(io::stdin())),
            _ => builder.from_path(filename)?,
        };

        for result in reader.byte_records() {
            writer.write_byte_record(&result?)?;
        }
    }

    writer.finish()?;
    Ok(())
}
