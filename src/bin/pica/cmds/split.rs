use crate::config::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::Arg;
use pica::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::fs::create_dir;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SplitConfig {
    pub skip_invalid: Option<bool>,
    pub gzip: Option<bool>,
    pub template: Option<String>,
}

pub fn cli() -> App {
    App::new("split")
        .about("Splits a stream of records into chunks.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("gzip")
                .short('g')
                .long("gzip")
                .about("compress output with gzip"),
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
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = match args.is_present("skip-invalid") {
        false => {
            if let Some(ref config) = config.split {
                config.skip_invalid.unwrap_or_default()
            } else if let Some(ref config) = config.global {
                config.skip_invalid.unwrap_or_default()
            } else {
                false
            }
        }
        _ => true,
    };

    let gzip_compression = match args.is_present("gzip") {
        false => {
            if let Some(ref config) = config.split {
                config.gzip.unwrap_or_default()
            } else {
                false
            }
        }
        _ => true,
    };

    let config_template = if let Some(ref config) = config.split {
        config
            .template
            .as_ref()
            .map(|x| x.to_owned())
            .unwrap_or_default()
    } else {
        String::new()
    };

    let filename_template = if args.is_present("template") {
        args.value_of("template").unwrap()
    } else if !config_template.is_empty() {
        &config_template
    } else if gzip_compression {
        "{}.dat.gz"
    } else {
        "{}.dat"
    };

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .from_path_or_stdin(args.value_of("filename"))?;

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

    let mut writer = WriterBuilder::new().gzip(gzip_compression).from_path(
        outdir
            .join(filename_template.replace("{}", &chunks.to_string()))
            .to_str()
            .unwrap(),
    )?;

    for (count, result) in reader.byte_records().enumerate() {
        let record = result?;

        if count > 0 && count as u32 % chunk_size == 0 {
            writer.finish()?;
            chunks += 1;

            writer = WriterBuilder::new().gzip(gzip_compression).from_path(
                outdir
                    .join(filename_template.replace("{}", &chunks.to_string()))
                    .to_str()
                    .unwrap(),
            )?;
        }

        writer.write_byte_record(&record)?;
    }

    writer.finish()?;
    Ok(())
}
