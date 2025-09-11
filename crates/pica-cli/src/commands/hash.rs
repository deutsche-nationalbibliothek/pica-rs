use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;
use csv::WriterBuilder;
use pica_record::prelude::*;

use crate::prelude::*;

/// Compute SHA-256 checksum of records.
#[derive(Parser, Debug)]
pub(crate) struct Hash {
    /// Comma-separated list of column names.
    #[arg(long, short = 'H', default_value = "ppn,hash")]
    header: String,

    /// Write output tab-separated (TSV)
    #[arg(long, short)]
    tsv: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to OUTPUT instead of stdout
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    #[command(flatten, next_help_heading = "Filter options")]
    filter_opts: FilterOpts,
}

impl Hash {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let options = MatcherOptions::from(&self.filter_opts);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;

        let writer: Box<dyn Write> = match self.output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout()),
        };

        let mut writer = WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer);

        let mut count = 0;

        writer.write_record(self.header.split(',').map(str::trim))?;

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

                        if !filter_set.check(&record) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher
                            && !matcher.is_match(&record, &options)
                        {
                            continue;
                        }

                        let hash = record.sha256().iter().fold(
                            String::new(),
                            |mut out, b| {
                                let _ = write!(out, "{b:02x}");
                                out
                            },
                        );

                        writer.write_record(&[
                            record
                                .ppn()
                                .unwrap_or_default()
                                .to_string(),
                            hash,
                        ])?;

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

        progress.finish();
        writer.flush()?;

        Ok(ExitCode::SUCCESS)
    }
}
