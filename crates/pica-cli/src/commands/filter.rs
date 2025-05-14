use std::ffi::OsString;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, value_parser};
use pica_record::prelude::*;

use crate::prelude::*;

/// Filter records by whether the given filter expression matches
#[derive(Parser, Debug)]
pub(crate) struct Filter {
    /// Whether to skip invalid records or not.
    #[arg(short, long)]
    skip_invalid: bool,

    /// Filter only records that did not match
    #[arg(long, short = 'v')]
    invert_match: bool,

    /// When this flag is provided, comparison operations will be
    /// search case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons
    /// (range: 0.0..1.0)
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
        default_value = "75")]
    strsim_threshold: u8,

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

    /// Ignore records which are *not* explicitly listed in one of the
    /// given allow-lists.
    ///
    /// An allow-list must be a CSV, TSV or Apache Arrow file. By
    /// default the column `ppn` or `idn` is used to get the values
    /// of the allow list. These values are compared against the PPN
    /// (003@.0) of record.
    ///
    /// The column name can be changed using the `--filter-set-column`
    /// option and the path to the comparison values can be changed
    /// with option `--filter-set-source`.
    ///
    /// # Note
    ///
    /// If the allow list is empty, all records are blocked. With more
    /// than one allow list, the filter set is made up of all partial
    /// lists. lists.
    #[arg(long = "allow-list", short = 'A')]
    allow: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists.
    ///
    /// A deny-list must be a CSV, TSV or Apache Arrow file. By
    /// default the column `ppn` or `idn` is used to get the values
    /// of the allow list. These values are compared against the PPN
    /// (003@.0) of record.
    ///
    /// The column name can be changed using the `--filter-set-column`
    /// option and the path to the comparison values can be changed
    /// with option `--filter-set-source`.
    ///
    /// # Note
    ///
    /// With more than one allow list, the filter set is made up of all
    /// partial lists.
    #[arg(long = "deny-list", short = 'D')]
    deny: Vec<PathBuf>,

    /// Defines the column name of an allow-list or a deny-list. By
    /// default, the column `ppn` is used or, if this is not
    /// available, the column `idn` is used.
    #[arg(long, value_name = "COLUMN")]
    filter_set_column: Option<String>,

    /// Defines an optional path to the comparison values of the
    /// record. If no path is specified, a comparison with the PPN in
    /// field 003@.0 is assumed.
    #[arg(long, value_name = "PATH")]
    filter_set_source: Option<Path>,

    /// Limit the result to first N records
    ///
    /// Note: A limit value `0` means no limit.
    #[arg(long, short, value_name = "N", default_value = "0")]
    limit: usize,

    /// Connects the filter with additional expressions using the
    /// logical AND-operator (conjunction)
    ///
    /// This option can't be combined with `--or`.
    #[arg(long, conflicts_with = "or")]
    and: Vec<String>,

    /// Connects the filter with additional expressions using the
    /// logical OR-operator (disjunction)
    ///
    /// This option can't be combined with `--and` or `--not`.
    #[arg(long, conflicts_with_all = ["and", "not"])]
    or: Vec<String>,

    /// Connects the filter with additional expressions using the
    /// logical NOT-operator (negation)
    ///
    /// This option can't be combined with `--or`.
    #[arg(long, conflicts_with = "or")]
    not: Vec<String>,

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
}

impl Filter {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let translit = translit(config.normalization.clone());
        let discard = parse_predicates(self.discard)?;
        let keep = parse_predicates(self.keep)?;

        let filter_set = FilterSetBuilder::new()
            .source(self.filter_set_source)
            .column(self.filter_set_column)
            .allow(self.allow)
            .deny(self.deny)
            .build()?;

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

        let matcher =
            RecordMatcherBuilder::with_transform(filter_str, translit)?
                .and(self.and)?
                .or(self.or)?
                .not(self.not)?
                .build();

        let mut count = 0;
        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100.0)
            .case_ignore(self.ignore_case);

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
                        if self.limit > 0 && count >= self.limit {
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
