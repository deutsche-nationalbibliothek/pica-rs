use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::create_dir;
use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;

use bstr::ByteSlice;
use clap::Arg;
use pica_api::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::{CliArgs, CliResult, Command};
use crate::{gzip_flag, skip_invalid_flag, template_opt};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct PartitionConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
    pub(crate) template: Option<String>,
}

pub(crate) fn cli() -> Command {
    Command::new("partition")
        .about("Partition a list of records by subfield value.")
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
        .arg(Arg::new("path").required(true))
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
    let skip_invalid =
        skip_invalid_flag!(args, config.partition, config.global);
    let gzip_compression = gzip_flag!(args, config.partition);
    let filename_template = template_opt!(
        args,
        config.partition,
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

    let mut writers: HashMap<Vec<u8>, Box<dyn PicaWriter>> = HashMap::new();
    let path = pica::Path::from_str(args.value_of("path").unwrap())?;

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

            let mut values = record.path(&path);
            values.sort_unstable();
            values.dedup();

            for value in values {
                let mut entry = writers.entry(value.as_bytes().to_vec());
                let writer = match entry {
                    Entry::Vacant(vacant) => {
                        let value = String::from_utf8(value.to_vec()).unwrap();
                        let writer = WriterBuilder::new()
                            .gzip(gzip_compression)
                            .from_path(
                                outdir
                                    .join(
                                        filename_template.replace("{}", &value),
                                    )
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
    }

    for (_, mut writer) in writers {
        writer.finish()?;
    }

    Ok(())
}
