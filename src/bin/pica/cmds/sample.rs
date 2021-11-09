use crate::config::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use crate::{gzip_flag, skip_invalid_flag};
use clap::Arg;
use pica::{ByteRecord, PicaWriter, ReaderBuilder, WriterBuilder};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SampleConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> App {
    App::new("sample")
        .about("Selects a random permutation of records")
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
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("sample-size").required(true))
        .arg(Arg::new("filename"))
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.sample, config.global);
    let gzip_compression = gzip_flag!(args, config.sample);

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .from_path_or_stdin(args.value_of("filename"))?;

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

    for (i, result) in reader.byte_records().enumerate() {
        let record = result?;

        if i < n {
            reservoir.push(record);
        } else {
            let j = rng.gen_range(0..i);
            if j < n {
                reservoir[j] = record;
            }
        }
    }

    for record in &reservoir {
        writer.write_byte_record(record)?;
    }

    writer.finish()?;
    Ok(())
}
