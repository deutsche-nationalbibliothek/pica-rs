use std::ffi::OsString;
use std::process::ExitCode;

use clap::{value_parser, Parser};
use pica_record::prelude::*;
use rand::rngs::StdRng;
use rand::{rng, Rng, SeedableRng};

use crate::config::Config;
use crate::error::CliResult;
use crate::progress::Progress;

/// Selects a random permutation of records of the given sample size
/// using reservoir sampling.
#[derive(Parser, Debug)]
pub(crate) struct Sample {
    /// Whether to skip invalid records or not
    #[arg(short, long)]
    skip_invalid: bool,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Initialize the RNG with a seed value to get deterministic
    /// random records.
    #[arg(long, value_name = "number")]
    seed: Option<u64>,

    /// Number of random records
    #[arg(value_parser = value_parser!(u16).range(1..))]
    sample_size: u16,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Sample {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut writer = WriterBuilder::new()
            .gzip(self.gzip)
            .from_path_or_stdout(self.output)?;

        let mut rng: StdRng = match self.seed {
            None => StdRng::from_rng(&mut rng()),
            Some(seed) => StdRng::seed_from_u64(seed),
        };

        let sample_size = self.sample_size as usize;
        let mut reservoir: Vec<Vec<u8>> =
            Vec::with_capacity(sample_size);

        let mut progress = Progress::new(self.progress);
        let mut i = 0;

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next_byte_record() {
                match result {
                    Err(e) if e.skip_parse_err(skip_invalid) => {
                        progress.update(true);
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                    Ok(ref record) => {
                        progress.update(false);
                        let mut data = Vec::<u8>::new();
                        record.write_to(&mut data)?;

                        if i < sample_size {
                            reservoir.push(data);
                        } else {
                            let j = rng.random_range(0..i);
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
            let record = ByteRecord::from_bytes(data).unwrap();
            writer.write_byte_record(&record)?;
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
