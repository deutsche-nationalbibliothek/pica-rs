use std::ffi::OsString;

use clap::{value_parser, Parser};
use pica_record::io::{ReaderBuilder, RecordsIterator, WriterBuilder};
use pica_record::ByteRecord;
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

/// Selects a random permutation of records
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

        let mut writer = WriterBuilder::new()
            .gzip(gzip_compression)
            .from_path_or_stdout(self.output)?;

        let mut rng: StdRng = match self.seed {
            None => StdRng::from_rng(thread_rng()).unwrap(),
            Some(seed) => StdRng::seed_from_u64(seed),
        };

        let sample_size = self.sample_size as usize;
        let mut reservoir: Vec<Vec<u8>> =
            Vec::with_capacity(sample_size);

        let mut i = 0;

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                match result {
                    Err(e) => {
                        if e.is_invalid_record() && skip_invalid {
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                    Ok(record) => {
                        let mut data = Vec::<u8>::new();
                        record.write_to(&mut data)?;

                        if i < sample_size {
                            reservoir.push(data);
                        } else {
                            let j = rng.gen_range(0..i);
                            if j < sample_size {
                                reservoir[j] = data;
                            }
                        }

                        i += 1;
                    }
                }
            }
        }

        for data in &reservoir {
            let record = ByteRecord::from_bytes(&data).unwrap();
            writer.write_byte_record(&record)?;
        }

        writer.finish()?;
        Ok(())
    }
}
