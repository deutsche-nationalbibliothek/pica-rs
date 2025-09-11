use std::ffi::OsString;
use std::process::ExitCode;

use clap::{Parser, value_parser};
use pica_record::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng, rng};

use crate::prelude::*;

/// Selects a random permutation of records of the given sample size
/// using reservoir sampling.
#[derive(Parser, Debug)]
pub(crate) struct Sample {
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
    #[arg(value_parser = value_parser!(u32).range(1..))]
    sample_size: u32,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    #[command(flatten, next_help_heading = "Filter options")]
    filter_opts: FilterOpts,
}

impl Sample {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let sample_size = self.sample_size as usize;
        let mut count = 0;

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let options = MatcherOptions::from(&self.filter_opts);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;

        let mut writer = WriterBuilder::new()
            .gzip(self.gzip)
            .from_path_or_stdout(self.output)?;

        let mut rng: StdRng = match self.seed {
            None => StdRng::from_rng(&mut rng()),
            Some(seed) => StdRng::seed_from_u64(seed),
        };

        let mut reservoir: Vec<Vec<u8>> =
            Vec::with_capacity(sample_size);

        'outer: for filename in self.filenames {
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

                        if !filter_set.check(record) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher
                            && !matcher.is_match(record, &options)
                        {
                            continue;
                        }

                        let mut data = Vec::<u8>::new();
                        record.write_to(&mut data)?;

                        if count < sample_size {
                            reservoir.push(data);
                        } else {
                            let j = rng.random_range(0..count);
                            if j < sample_size {
                                reservoir[j] = data;
                            }
                        }

                        count += 1;
                        if self.filter_opts.limit > 0
                            && count >= self.filter_opts.limit
                        {
                            break 'outer;
                        }
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
