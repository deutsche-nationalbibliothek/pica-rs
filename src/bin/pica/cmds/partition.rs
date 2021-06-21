use crate::config::Config;
use crate::util::{App, CliArgs, CliResult};
use bstr::ByteSlice;
use clap::Arg;
use pica::{self, PicaWriter, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::create_dir;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartitionConfig {
    pub skip_invalid: Option<bool>,
    pub gzip: Option<bool>,
    pub template: Option<String>,
}

pub fn cli() -> App {
    App::new("partition")
        .about("Partition a list of records by subfield value.")
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
        .arg(Arg::new("path").required(true))
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = match args.is_present("skip-invalid") {
        false => {
            if let Some(ref partition_config) = config.partition {
                partition_config.skip_invalid.unwrap_or_default()
            } else if let Some(ref global_config) = config.global {
                global_config.skip_invalid.unwrap_or_default()
            } else {
                false
            }
        }
        _ => true,
    };

    let gzip_compression = match args.is_present("gzip") {
        false => {
            if let Some(ref partition_config) = config.partition {
                partition_config.gzip.unwrap_or_default()
            } else {
                false
            }
        }
        _ => true,
    };

    let config_template = if let Some(ref partition_config) = config.partition {
        partition_config
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

    let mut writers: HashMap<Vec<u8>, Box<dyn PicaWriter>> = HashMap::new();
    let path = pica::Path::from_str(args.value_of("path").unwrap())?;

    for result in reader.byte_records() {
        let record = result?;

        for value in record.path(&path) {
            let mut entry = writers.entry(value.as_bytes().to_vec());
            let writer = match entry {
                Entry::Vacant(vacant) => {
                    let value = String::from_utf8(value.to_vec()).unwrap();
                    let writer =
                        WriterBuilder::new().gzip(gzip_compression).from_path(
                            outdir
                                .join(filename_template.replace("{}", &value))
                                .to_str()
                                .unwrap(),
                        )?;

                    vacant.insert(writer)
                }
                Entry::Occupied(ref mut occupied) => occupied.get_mut(),
            };

            writer.write_byte_record(&record)?;
        }
    }

    for (_, mut writer) in writers {
        writer.finish()?;
    }

    Ok(())
}
