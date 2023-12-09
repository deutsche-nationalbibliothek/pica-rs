use std::ffi::OsString;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{value_parser, Parser};
use pica_matcher::{
    MatcherBuilder, MatcherOptions, OccurrenceMatcher,
    ParseMatcherError, TagMatcher,
};
use pica_path::PathExt;
use pica_record::io::{ReaderBuilder, RecordsIterator, WriterBuilder};
use pica_utils::FilterList;
use serde::{Deserialize, Serialize};

use crate::error::CliResult;
use crate::progress::Progress;
use crate::{gzip_flag, skip_invalid_flag, Config};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct FilterConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

/// Filter records by whether the given filter expression matches
#[derive(Parser, Debug)]
pub(crate) struct Filter {
    /// Skip invalid records that can't be decoded as normalized PICA+
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

    /// Take a filter expression from <EXPR_FILE>
    ///
    /// Note: Do not provide an additional filter expression as an CLI
    /// argument!
    #[arg(long = "file", short = 'F')]
    expr_file: Option<PathBuf>,

    /// Ignore records which are *not* explicitly listed in one of the
    /// given allow-lists.
    ///
    /// An allow-list must be an CSV, whereby the first column contains
    /// the IDN (003@.0) or an Apache Arrow file with an `idn` column.
    /// If the file extension is `.feather`, `.arrow`, or `.ipc` the
    /// file is automatically interpreted as Apache Arrow;
    /// otherwise the file is read as CSV.
    #[arg(long = "allow-lists", short = 'A')]
    allow_lists: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists.
    ///
    /// An allow-list must be an CSV, whereby the first column contains
    /// the IDN (003@.0) or an Apache Arrow file with an `idn` column.
    /// If the file extension is `.feather`, `.arrow`, or `.ipc` the
    /// file is automatically interpreted as Apache Arrow;
    /// otherwise the file is read as CSV.
    #[arg(long = "deny-lists", short = 'D')]
    deny_lists: Vec<PathBuf>,

    /// Limit the result to first <n> records
    ///
    /// Note: A limit value `0` means no limit.
    #[arg(long, short, value_name = "n", default_value = "0")]
    limit: usize,

    /// Connects the filter with additional expressions using the
    /// logical AND-operator (conjunction)
    ///
    /// This option can't be combined with `--or` or `--not`.
    #[arg(long, conflicts_with_all = ["or", "not"])]
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
    /// This option can't be combined with `--and` or `--or`.
    #[arg(long, conflicts_with_all = ["and", "or"])]
    not: Vec<String>,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Append to the given file, do not overwrite
    ///
    /// Warning: This option can't be used when writing to a gzip file.
    #[arg(long, conflicts_with = "gzip")]
    append: bool,

    /// Write simultaneously to the file <filename> and stdout
    #[arg(long, value_name = "filename", conflicts_with = "output")]
    tee: Option<PathBuf>,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
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
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let gzip_compression = gzip_flag!(self.gzip, config.filter);
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.filter,
            config.global
        );

        let nf = if let Some(ref global) = config.global {
            global.translit
        } else {
            None
        };

        let mut writer = WriterBuilder::new()
            .gzip(gzip_compression)
            .append(self.append)
            .from_path_or_stdout(self.output)?;

        let mut tee_writer = match self.tee {
            Some(path) => Some(
                WriterBuilder::new()
                    .gzip(gzip_compression)
                    .append(self.append)
                    .from_path(path)?,
            ),
            None => None,
        };

        let discard = self.discard.unwrap_or_default();
        let discard_predicates = parse_predicates(&discard)?;

        let keep = self.keep.unwrap_or_default();
        let keep_predicates = parse_predicates(&keep)?;

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

            read_to_string(filename).unwrap()
        } else {
            self.filter
        };

        let filter = MatcherBuilder::new(filter_str, nf)?
            .and(self.and)?
            .or(self.or)?
            .not(self.not)?
            .build();

        let filter_list = FilterList::new()
            .allow(self.allow_lists)
            .unwrap()
            .deny(self.deny_lists)
            .unwrap();

        let mut progress = Progress::new(self.progress);

        let mut count = 0;
        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100.0)
            .case_ignore(self.ignore_case);

        'outer: for filename in filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                if let Err(e) = result {
                    if e.is_invalid_record() && skip_invalid {
                        progress.invalid();
                        continue;
                    } else {
                        return Err(e.into());
                    }
                }

                let mut record = result.unwrap();
                progress.record();

                if !filter_list.check(record.idn()) {
                    continue;
                }

                let mut is_match = filter.is_match(&record, &options);
                if self.invert_match {
                    is_match = !is_match;
                }

                if !is_match {
                    continue;
                }

                if !keep_predicates.is_empty() {
                    record.retain(|field| {
                        for (t, o) in keep_predicates.iter() {
                            if t.is_match(field.tag())
                                && *o == field.occurrence()
                            {
                                return true;
                            }
                        }
                        false
                    });
                }

                if !discard_predicates.is_empty() {
                    record.retain(|field| {
                        for (t, o) in discard_predicates.iter() {
                            if t.is_match(field.tag())
                                && *o == field.occurrence()
                            {
                                return false;
                            }
                        }
                        true
                    });
                }

                writer.write_byte_record(&record)?;
                if let Some(ref mut writer) = tee_writer {
                    writer.write_byte_record(&record)?;
                }

                count += 1;

                if self.limit > 0 && count >= self.limit {
                    break 'outer;
                }
            }
        }

        if let Some(ref mut writer) = tee_writer {
            writer.finish()?;
        }

        progress.finish();
        writer.finish()?;
        Ok(())
    }
}

fn parse_predicates(
    s: &str,
) -> Result<Vec<(TagMatcher, OccurrenceMatcher)>, ParseMatcherError> {
    let items = s.split(',').map(str::trim).filter(|s| !s.is_empty());
    let mut result = vec![];

    for item in items {
        if let Some(pos) = item.rfind('/') {
            result.push((
                TagMatcher::from_str(&item[0..pos])?,
                OccurrenceMatcher::from_str(&item[pos..])?,
            ));
        } else {
            result.push((
                TagMatcher::from_str(item)?,
                OccurrenceMatcher::None,
            ));
        }
    }

    Ok(result)
}
