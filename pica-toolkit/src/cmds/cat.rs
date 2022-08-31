use std::ffi::OsString;
use std::io::{self, Read};

use crate::config::Config;
use crate::util::{CliArgs, CliResult, Command};
use crate::{gzip_flag, skip_invalid_flag};
use clap::Arg;
use pica_api::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
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
                .help("Whether to skip invalid records that can't be decoded.")
                .long("skip-invalid")
                .short('s')
        )
        .arg(
            Arg::new("gzip")
                .help("If the --gzip flag is provided the output will be compressed \
                      and written in gzip file format.")
                .long("gzip")
                .short('g')
                .requires("output"),
        )
        .arg(
            Arg::new("append")
                .long("--append")
                .help("Append to the given <file>, do not overwrite.")
       )
        .arg(
            Arg::new("tee")
            .help(
                "This option allows to write simultaneously to <file> and to \
                standard output (stdout)."
            )
            .long("--tee")
            .value_name("filename")
        )
        .arg(
            Arg::new("output")
                .help(
                    "Write output to <filename> instead of stdout. If the file ends with \
                    .gz the file content will be compressed and written in gzip file\
                     format.")
                .value_name("filename")
                .long("--output")
                .short('o')
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
    let skip_invalid = skip_invalid_flag!(args, config.cat, config.global);
    let gzip_compression = gzip_flag!(args, config.cat);
    let append = args.is_present("append");

    let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
        .gzip(gzip_compression)
        .append(append)
        .from_path_or_stdout(args.value_of("output"))?;

    let mut tee_writer = match args.value_of("tee") {
        Some(path) => Some(
            WriterBuilder::new()
                .gzip(gzip_compression)
                .append(append)
                .from_path(path)?,
        ),
        None => None,
    };

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
            let record = result?;

            writer.write_byte_record(&record)?;

            if let Some(ref mut writer) = tee_writer {
                writer.write_byte_record(&record)?;
            }
        }
    }

    writer.finish()?;

    if let Some(ref mut writer) = tee_writer {
        writer.finish()?;
    }

    Ok(())
}
