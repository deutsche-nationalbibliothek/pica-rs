use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use pica_matcher::{MatcherOptions, RecordMatcher};
use pica_record::io::{ReaderBuilder, RecordsIterator};
use pica_select::{Query, QueryExt, QueryOptions};
use serde::{Deserialize, Serialize};

use crate::common::FilterList;
use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::{translit_maybe, translit_maybe2};
use crate::util::CliResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SelectConfig {
    pub(crate) skip_invalid: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Select {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Squash subfield values.
    #[arg(long)]
    squash: bool,

    #[arg(long, default_value = "|")]
    separator: String,

    /// Disallow empty columns
    #[arg(long)]
    no_empty_columns: bool,

    /// Skip duplicate rows
    #[arg(long, short)]
    unique: bool,

    /// When this flag is provided, comparision operations will be
    /// search case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// Write output tab-separated (TSV)
    #[arg(long, short)]
    tsv: bool,

    /// Transliterate output into the selected normalform <NF>
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc")
    #[arg(long,
          value_name = "NF",
          value_parser = ["nfd", "nfkd", "nfc", "nfkc"],
          hide_possible_values = true,
    )]
    translit: Option<String>,

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
    /// file is automatically interpreted as Apachae Arrow;
    /// otherwise the file is read as CSV.
    #[arg(long, short = 'A')]
    allow_list: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists.
    ///
    /// An allow-list must be an CSV, whereby the first column contains
    /// the IDN (003@.0) or an Apache Arrow file with an `idn` column.
    /// If the file extension is `.feather`, `.arrow`, or `.ipc` the
    /// file is automatically interpreted as Apachae Arrow;
    /// otherwise the file is read as CSV.
    #[arg(long, short = 'D')]
    deny_list: Vec<PathBuf>,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Query (comma-separated list of path expressions or string
    /// literals)
    query: String,

    /// Read one or more files in normalized PICA+ format.
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
        let translit = if let Some(ref global) = config.global {
            global.translit
        } else {
            None
        };

        let options = QueryOptions::default()
            .case_ignore(self.ignore_case)
            .separator(self.separator)
            .squash(self.squash);

        let matcher = if let Some(matcher_str) = self.filter {
            let mut matcher = RecordMatcher::new(&translit_maybe2(
                &matcher_str,
                translit,
            ))?;

            for matcher_str in self.and.iter() {
                matcher = matcher
                    & RecordMatcher::new(&translit_maybe2(
                        matcher_str,
                        translit,
                    ))?;
            }

            for matcher_str in self.or.iter() {
                matcher = matcher
                    | RecordMatcher::new(&translit_maybe2(
                        matcher_str,
                        translit,
                    ))?;
            }

            for matcher_str in self.not.iter() {
                matcher = matcher
                    & !RecordMatcher::new(&translit_maybe2(
                        matcher_str,
                        translit,
                    ))?;
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

        let query = if let Some(ref global) = config.global {
            Query::from_str(&translit_maybe2(
                &self.query,
                global.translit,
            ))?
        } else {
            Query::from_str(&self.query)?
        };

        let mut writer = csv::WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer(self.output, self.append)?);

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(|s| s.trim()))?;
        }

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
                                && row
                                    .iter()
                                    .any(|column| column.is_empty())
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

                            if !row.iter().all(|col| col.is_empty()) {
                                if self.translit.is_some() {
                                    writer.write_record(
                                        row.iter().map(|s| {
                                            translit_maybe(
                                                s,
                                                self.translit
                                                    .as_deref(),
                                            )
                                        }),
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

        writer.flush()?;
        Ok(())
    }
}
