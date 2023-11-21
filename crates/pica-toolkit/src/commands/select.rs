use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::path::PathBuf;

use clap::Parser;
use pica_matcher::{MatcherOptions, RecordMatcher};
use pica_record::io::{ReaderBuilder, RecordsIterator};
use pica_select::{Query, QueryExt, QueryOptions};
use pica_utils::NormalizationForm;
use serde::{Deserialize, Serialize};

use crate::common::FilterList;
use crate::config::Config;
use crate::progress::Progress;
use crate::skip_invalid_flag;
use crate::util::CliResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SelectConfig {
    pub(crate) skip_invalid: Option<bool>,
}

/// Select subfield values from records
#[derive(Parser, Debug)]
pub(crate) struct Select {
    /// Skip invalid records that can't be decoded as normalized PICA+
    #[arg(short, long)]
    skip_invalid: bool,

    /// Whether to squash all values of a repeated subfield into a
    /// single value or not. The separator can be specified by the
    /// `--separator` option.
    ///
    /// Note: This option cannot be used with `--merge`.
    #[arg(long, conflicts_with = "merge")]
    squash: bool,

    /// Whether to merge all values of a column into a single value or
    /// not. The separator can be specified by the `--separator`
    /// Note: This option cannot be used with `--merge`.
    /// option.
    ///
    /// Note: This option cannot be used with `--squash`.
    #[arg(long, conflicts_with = "squash")]
    merge: bool,

    /// Sets the separator used for squashing of repeated subfield
    /// values into a single value. Note that it's possible to use the
    /// empty string as a separator.
    #[arg(long, default_value = "|")]
    separator: String,

    /// Disallow empty columns
    #[arg(long)]
    no_empty_columns: bool,

    /// Skip duplicate rows
    #[arg(long, short)]
    unique: bool,

    /// When this flag is provided, comparison operations will be
    /// search case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// Write output tab-separated (TSV)
    #[arg(long, short)]
    tsv: bool,

    /// Transliterate output into the selected normal form <NF>
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc")
    #[arg(long = "translit", value_name = "NF")]
    nf: Option<NormalizationForm>,

    /// Comma-separated list of column names
    #[arg(long, short = 'H')]
    header: Option<String>,

    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// A filter expression used for searching
    #[arg(long = "where")]
    filter: Option<String>,

    /// Connects the where clause with additional expressions using the
    /// logical AND-operator (conjunction)
    ///
    /// This option can't be combined with `--or` or `--not`.
    #[arg(long, requires = "filter", conflicts_with_all = ["or", "not"])]
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
    /// This option can't be combined with `--and` or `--or`.
    #[arg(long, requires = "filter", conflicts_with_all = ["and", "or"])]
    not: Vec<String>,

    /// Ignore records which are *not* explicitly listed in one of the
    /// given allow-lists.
    ///
    /// An allow-list must be an CSV, whereby the first column contains
    /// the IDN (003@.0) or an Apache Arrow file with an `idn` column.
    /// If the file extension is `.feather`, `.arrow`, or `.ipc` the
    /// file is automatically interpreted as Apache Arrow;
    /// otherwise the file is read as CSV.
    #[arg(long, short = 'A')]
    allow_list: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists.
    ///
    /// An allow-list must be an CSV, whereby the first column contains
    /// the IDN (003@.0) or an Apache Arrow file with an `idn` column.
    /// If the file extension is `.feather`, `.arrow`, or `.ipc` the
    /// file is automatically interpreted as Apache Arrow;
    /// otherwise the file is read as CSV.
    #[arg(long, short = 'D')]
    deny_list: Vec<PathBuf>,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Query (comma-separated list of path expressions or string
    /// literals)
    query: String,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

fn writer(
    filename: Option<OsString>,
    append: bool,
) -> CliResult<Box<dyn Write>> {
    Ok(match filename {
        Some(filename) => Box::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(!append)
                .append(append)
                .open(filename)?,
        ),
        None => Box::new(io::stdout()),
    })
}

impl Select {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.select,
            config.global
        );

        let mut seen = BTreeSet::new();
        let nf = if let Some(ref global) = config.global {
            global.translit
        } else {
            None
        };

        let options = QueryOptions::default()
            .case_ignore(self.ignore_case)
            .separator(self.separator)
            .squash(self.squash)
            .merge(self.merge);

        let filter = self
            .filter
            .map(|value| NormalizationForm::translit_opt(value, nf));

        let and: Vec<String> = self
            .and
            .iter()
            .map(|value| NormalizationForm::translit_opt(value, nf))
            .collect();

        let or: Vec<String> = self
            .or
            .iter()
            .map(|value| NormalizationForm::translit_opt(value, nf))
            .collect();

        let not: Vec<String> = self
            .not
            .iter()
            .map(|value| NormalizationForm::translit_opt(value, nf))
            .collect();

        let matcher = if let Some(ref matcher_str) = filter {
            let mut matcher = RecordMatcher::try_from(matcher_str)?;

            for matcher_str in and.iter() {
                matcher =
                    matcher & RecordMatcher::try_from(matcher_str)?;
            }

            for matcher_str in or.iter() {
                matcher =
                    matcher | RecordMatcher::try_from(matcher_str)?;
            }

            for matcher_str in not.iter() {
                matcher =
                    matcher & !RecordMatcher::try_from(matcher_str)?;
            }

            Some(matcher)
        } else {
            None
        };

        let allow_list = if !self.allow_list.is_empty() {
            FilterList::new(self.allow_list)?
        } else {
            FilterList::default()
        };

        let deny_list = if !self.deny_list.is_empty() {
            FilterList::new(self.deny_list)?
        } else {
            FilterList::default()
        };

        let query = NormalizationForm::translit_opt(&self.query, nf);
        let query = Query::try_from(&query)?;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer(self.output, self.append)?);

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(str::trim))?;
        }

        let mut progess = Progress::new(self.progress);

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                match result {
                    Err(e) => {
                        if e.is_invalid_record() && skip_invalid {
                            progess.invalid();
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                    Ok(record) => {
                        progess.record();

                        if !allow_list.is_empty()
                            && !allow_list.check(&record)
                        {
                            continue;
                        }

                        if !deny_list.is_empty()
                            && deny_list.check(&record)
                        {
                            continue;
                        }

                        if let Some(ref matcher) = matcher {
                            if !matcher.is_match(
                                &record,
                                &MatcherOptions::from(&options),
                            ) {
                                continue;
                            }
                        }

                        let outcome = record.query(&query, &options);
                        for row in outcome.iter() {
                            if self.no_empty_columns
                                && row.iter().any(String::is_empty)
                            {
                                continue;
                            }

                            if self.unique {
                                let mut hasher = DefaultHasher::new();
                                row.hash(&mut hasher);
                                let hash = hasher.finish();

                                if seen.contains(&hash) {
                                    continue;
                                }

                                seen.insert(hash);
                            }

                            if !row.iter().all(String::is_empty) {
                                if let Some(nf) = self.nf {
                                    writer.write_record(
                                        row.iter()
                                            .map(|s| nf.translit(s)),
                                    )?;
                                } else {
                                    writer.write_record(row)?;
                                };
                            }
                        }
                    }
                }
            }
        }

        progess.finish();
        writer.flush()?;
        Ok(())
    }
}
