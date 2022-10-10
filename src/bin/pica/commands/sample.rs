use std::ffi::OsString;
use std::io::{self, Read};

use clap::Arg;
use pica::{
    ByteRecord, PicaWriter, Reader, ReaderBuilder, WriterBuilder,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::{CliArgs, CliError, CliResult, Command};
use crate::{gzip_flag, skip_invalid_flag};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SampleConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> Command {
    Command::new("sample")
        .about("Selects a random permutation of records")
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
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("sample-size").required(true))
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
        skip_invalid_flag!(args, config.sample, config.global);
    let gzip_compression = gzip_flag!(args, config.sample);

    let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
        .gzip(gzip_compression)
        .from_path_or_stdout(args.value_of("output"))?;

    let sample_size = args.value_of("sample-size").unwrap();
    let n = match sample_size.parse::<usize>() {
        Err(_) | Ok(0) => {
            return Err(CliError::Other(format!(
                "invalid sample size '{}'. expected non-zero usize.",
                sample_size
            )));
        }
        Ok(v) => v,
    };

    let mut reservoir: Vec<ByteRecord> = Vec::with_capacity(n);
    let mut rng = thread_rng();
    let mut i = 0;

    let filenames = args
        .values_of_t::<OsString>("filenames")
        .unwrap_or_else(|_| vec![OsString::from("-")]);

    for filename in filenames {
        let builder = ReaderBuilder::new().skip_invalid(skip_invalid);
        let mut reader: Reader<Box<dyn Read>> = match filename.to_str()
        {
            Some("-") => builder.from_reader(Box::new(io::stdin())),
            _ => builder.from_path(filename)?,
        };

        for result in reader.byte_records() {
            let record = result?;

            if i < n {
                reservoir.push(record);
            } else {
                let j = rng.gen_range(0..i);
                if j < n {
                    reservoir[j] = record;
                }
            }

            i += 1;
        }
    }

    for record in &reservoir {
        writer.write_byte_record(record)?;
    }

    writer.finish()?;
    Ok(())
}
