use std::ffi::OsString;
use std::path::PathBuf;
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

    /// When this flag is provided, comparison operations will be
    /// search case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons (0 <= score
    /// < 100).
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
          default_value = "75")]
    strsim_threshold: u8,

    /// A filter expression used for searching
    #[arg(long = "where")]
    filter: Option<String>,

    /// Connects the where clause with additional expressions using the
    /// logical AND-operator (conjunction)
    ///
    /// This option can't be combined with `--or`.
    #[arg(long, requires = "filter", conflicts_with = "or")]
    and: Vec<String>,

    /// Connects the where clause with additional expressions using the
    /// logical OR-operator (disjunction)
    ///
    /// This option can't be combined with `--and` or `--not`.
    #[arg(long, requires = "filter", conflicts_with_all = ["and", "not"])]
    or: Vec<String>,

    /// Connects the where clause with additional expressions using the
    /// logical NOT-operator (negation)
    ///
    /// This option can't be combined with `--or`.
    #[arg(long, requires = "filter", conflicts_with = "or")]
    not: Vec<String>,

    /// Ignore records which are *not* explicitly listed in one of the
    /// given allow-lists.
    ///
    /// A allow-list must be an CSV/TSV or Apache Arrow file, whereby
    /// a column `idn` exists. If the file extension is `.feather`,
    /// `.arrow`, or `.ipc` the file is automatically interpreted
    /// as Apache Arrow; file existions `.csv`, `.csv.gz`, `.tsv` or
    /// `.tsv.gz` is interpreted as CSV/TSV.
    #[arg(long = "allow-list", short = 'A')]
    allow: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists.
    ///
    /// A deny-list must be an CSV/TSV or Apache Arrow file, whereby
    /// a column `idn` exists. If the file extension is `.feather`,
    /// `.arrow`, or `.ipc` the file is automatically interpreted
    /// as Apache Arrow; file existions `.csv`, `.csv.gz`, `.tsv` or
    /// `.tsv.gz` is interpreted as CSV/TSV.
    #[arg(long = "deny-list", short = 'D')]
    deny: Vec<PathBuf>,

    #[arg(long, value_name = "PATH")]
    filter_set_source: Option<Path>,

    #[arg(long, value_name = "COLUMN")]
    filter_set_column: Option<String>,

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

        let filter_set = FilterSetBuilder::new()
            .source(self.filter_set_source)
            .column(self.filter_set_column)
            .allow(self.allow)
            .deny(self.deny)
            .build()?;

        let mut rng: StdRng = match self.seed {
            None => StdRng::from_rng(&mut rng()),
            Some(seed) => StdRng::seed_from_u64(seed),
        };

        let sample_size = self.sample_size as usize;
        let mut reservoir: Vec<Vec<u8>> =
            Vec::with_capacity(sample_size);

        let matcher = if let Some(matcher) = self.filter {
            Some(
                RecordMatcherBuilder::with_transform(
                    matcher,
                    translit(config.normalization.clone()),
                )?
                .and(self.and)?
                .or(self.or)?
                .not(self.not)?
                .build(),
            )
        } else {
            None
        };

        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100.0)
            .case_ignore(self.ignore_case);

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

                        if !filter_set.check(record) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher {
                            if !matcher.is_match(record, &options) {
                                continue;
                            }
                        }

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
