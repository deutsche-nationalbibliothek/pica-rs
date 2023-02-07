use std::ffi::OsString;
use std::io::{self, Read};

use clap::{value_parser, Parser};
use pica::{
    ByteRecord, PicaWriter, Reader, ReaderBuilder, WriterBuilder,
};
use rand::rngs::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::CliResult;
use crate::{gzip_flag, skip_invalid_flag};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SampleConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Sample {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// RNG seed
    #[arg(long, value_name = "number")]
    seed: Option<u64>,

    #[arg(value_parser = value_parser!(u16).range(1..))]
    sample_size: u16,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Sample {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let gzip_compression = gzip_flag!(self.gzip, config.sample);
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.sample,
            config.global
        );

        let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
            .gzip(gzip_compression)
            .from_path_or_stdout(self.output)?;

        let sample_size = self.sample_size as usize;
        let mut reservoir: Vec<ByteRecord> =
            Vec::with_capacity(sample_size);

        let mut rng: StdRng = match self.seed {
            None => StdRng::from_rng(thread_rng()).unwrap(),
            Some(seed) => StdRng::seed_from_u64(seed),
        };

        let mut i = 0;

        for filename in self.filenames {
            let builder =
                ReaderBuilder::new().skip_invalid(skip_invalid);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for result in reader.byte_records() {
                let record = result?;

                if i < sample_size {
                    reservoir.push(record);
                } else {
                    let j = rng.gen_range(0..i);
                    if j < sample_size {
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
}
