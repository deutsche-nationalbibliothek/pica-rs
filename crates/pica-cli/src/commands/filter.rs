use std::ffi::OsString;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use pica_record::prelude::*;

use crate::cli::FilterOpts;
use crate::prelude::*;
use crate::utils::FilterSet;

/// Filter records by whether the given filter expression matches
#[derive(Parser, Debug)]
pub(crate) struct Filter {
    /// Filter only records that did not match
    #[arg(long, short = 'v')]
    invert_match: bool,

    /// Keep only fields specified by a list of predicates.
    #[arg(long, short)]
    keep: Option<String>,

    /// Discard fields specified by a list of predicates.
    #[arg(long, short)]
    discard: Option<String>,

    /// Take a filter expression from FILENAME
    ///
    /// Note: Do not provide an additional filter expression as an CLI
    /// argument!
    #[arg(long = "file", short = 'F', value_name = "FILENAME")]
    expr_file: Option<PathBuf>,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Append to the given file, do not overwrite
    ///
    /// Warning: This option can't be used when writing to a gzip file.
    #[arg(long, conflicts_with = "gzip")]
    append: bool,

    /// Write simultaneously to the file FILENAME and stdout
    #[arg(long, value_name = "FILENAME", conflicts_with = "output")]
    tee: Option<PathBuf>,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// A filter expression used for searching
    #[arg(default_value = "", hide_default_value = true)]
    filter: String,

    /// Read one or more files in normalized PICA+ format
    ///
    /// If no filenames where given or a filename is "-", data is read
    /// from standard input (stdin).
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    #[command(flatten, next_help_heading = "Filter options")]
    pub(crate) filter_opts: FilterOpts,
}

impl Filter {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let discard = parse_predicates(self.discard)?;
        let keep = parse_predicates(self.keep)?;

        let mut writer = WriterBuilder::new()
            .append(self.append)
            .gzip(self.gzip)
            .from_path_or_stdout(self.output)?;

        let mut tee_writer = match self.tee {
            Some(path) => Some(
                WriterBuilder::new()
                    .append(self.append)
                    .gzip(self.gzip)
                    .from_path(path)?,
            ),
            None => None,
        };

        let mut filenames = self.filenames;
        let filter_str = if let Some(filename) = self.expr_file {
            // This "hack" is necessary, because it's not possible to
            // distinguish between filter and filenames. If
            // a expression file is given, it makes no sense to provide
            // an filter expression as CLI argument.
            if !self.filter.is_empty() {
                if filenames != ["-"] {
                    filenames.insert(0, self.filter.into());
                } else {
                    filenames = vec![self.filter.into()];
                }
            }

            read_to_string(filename)?
        } else {
            self.filter
        };

        let options = MatcherOptions::from(&self.filter_opts);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), Some(filter_str))?
            .expect("filter expression");

        let mut count = 0;

        'outer: for path in filenames {
            let mut reader = ReaderBuilder::new().from_path(path)?;

            while let Some(mut result) = reader.next_byte_record() {
                match result {
                    Err(e) if e.skip_parse_err(skip_invalid) => {
                        progress.update(true);
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                    Ok(ref mut record) => {
                        progress.update(false);

                        if !filter_set.check(record) {
                            continue;
                        }

                        let mut is_match =
                            matcher.is_match(record, &options);
                        if self.invert_match {
                            is_match = !is_match;
                        }
                        if !is_match {
                            continue;
                        }

                        record.discard(&discard);
                        record.keep(&keep);

                        writer.write_byte_record(record)?;
                        if let Some(ref mut writer) = tee_writer {
                            writer.write_byte_record(record)?;
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

        if let Some(ref mut writer) = tee_writer {
            writer.finish()?;
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
