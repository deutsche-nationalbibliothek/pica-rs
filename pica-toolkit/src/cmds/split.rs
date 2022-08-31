use std::ffi::OsString;
use std::fs::create_dir;
use std::io::{self, Read};
use std::path::Path;

use clap::Arg;
use pica_api::{Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::{CliArgs, CliError, CliResult, Command};
use crate::{gzip_flag, skip_invalid_flag, template_opt};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SplitConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
    pub(crate) template: Option<String>,
}

pub(crate) fn cli() -> Command {
    Command::new("split")
        .about("Splits a stream of records into chunks.")
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
                .help("compress output with gzip"),
        )
        .arg(
            Arg::new("outdir")
                .short('o')
                .long("--outdir")
                .value_name("outdir")
                .default_value("."),
        )
        .arg(
            Arg::new("template")
                .short('t')
                .long("--template")
                .value_name("template"),
        )
        .arg(Arg::new("size").required(true))
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
    let skip_invalid = skip_invalid_flag!(args, config.split, config.global);
    let gzip_compression = gzip_flag!(args, config.split);
    let filename_template = template_opt!(
        args,
        config.split,
        if gzip_compression {
            "{}.dat.gz"
        } else {
            "{}.dat"
        }
    );

    let outdir = Path::new(args.value_of("outdir").unwrap());
    if !outdir.exists() {
        create_dir(outdir)?;
    }

    let chunk_size = args.value_of("size").unwrap_or("500");
    let chunk_size = match chunk_size.parse::<u32>() {
        Ok(size) => size,
        Err(_) => {
            return Err(CliError::Other("invalid chunk size".to_string()))
        }
    };

    if chunk_size == 0 {
        return Err(CliError::Other("chunk size < 1".to_string()));
    }

    let mut chunks: u32 = 0;
    let mut count = 0;

    let mut writer = WriterBuilder::new().gzip(gzip_compression).from_path(
        outdir
            .join(filename_template.replace("{}", &chunks.to_string()))
            .to_str()
            .unwrap(),
    )?;

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

            if count > 0 && count as u32 % chunk_size == 0 {
                writer.finish()?;
                chunks += 1;

                writer =
                    WriterBuilder::new().gzip(gzip_compression).from_path(
                        outdir
                            .join(
                                filename_template
                                    .replace("{}", &chunks.to_string()),
                            )
                            .to_str()
                            .unwrap(),
                    )?;
            }

            writer.write_byte_record(&record)?;
            count += 1;
        }
    }

    writer.finish()?;
    Ok(())
}
