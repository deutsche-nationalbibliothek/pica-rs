use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::str::FromStr;

use clap::{value_parser, Parser};
use pica_record::io::{ReaderBuilder, RecordsIterator};
use pica_select::{Query, QueryExt, QueryOptions};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::{translit_maybe, translit_maybe2};
use crate::util::CliResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct FrequencyConfig {
    pub(crate) skip_invalid: Option<bool>,
}

/// Compute a frequency table of a subfield
///
/// This command computes a frequency table over all subfield values of
/// the given path expression. By default, the resulting frequency table
/// is sorted in descending order by default (the most frequent value is
/// printed first). If the count of two or more subfield values is
/// equal, these lines are given in lexicographical order.
///
/// The set of fields, which are included in the result of a record, can
/// be restricted by an optional subfield filter. A subfield filter
/// requires the {}-notation and is expected at the first position (e.g.
/// "044H/*{b == 'GND' && 9?, 9}").
#[derive(Parser, Debug)]
pub(crate) struct Frequency {
    /// Skip invalid records that can't be decoded as normalized PICA+.
    #[arg(long, short)]
    skip_invalid: bool,

    /// When this flag is set, comparison operations will be search
    /// case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons (0 <= score
    /// < 100).
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
          default_value = "75")]
    strsim_threshold: u8,

    /// Sort results in reverse order.
    #[arg(long, short)]
    reverse: bool,

    /// Limit result to the <n> most frequent subfield values.
    #[arg(
        long,
        short,
        value_name = "n",
        hide_default_value = true,
        default_value = "0"
    )]
    limit: usize,

    /// Ignore rows with a frequency < <value>.
    #[arg(
        long,
        value_name = "value",
        default_value = "0",
        hide_default_value = true
    )]
    threshold: u64,

    /// Comma-separated list of column names.
    #[arg(long, short = 'H')]
    header: Option<String>,

    /// Write output tab-separated (TSV)
    #[arg(long, short)]
    tsv: bool,

    /// Transliterate output into the selected normal form <NF>
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc").
    #[arg(long,
          value_name = "NF",
          value_parser = ["nfd", "nfkd", "nfc", "nfkc"],
          hide_possible_values = true,
    )]
    translit: Option<String>,

    /// Write output to <filename> instead of stdout.
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// A PICA path expression
    query: String,

    /// Read one or more files in normalized PICA+ format. With no
    /// files, or when a filename is '-', read from standard input
    /// (stdin).
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Frequency {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.frequency,
            config.global
        );

        let query = if let Some(ref global) = config.global {
            Query::from_str(&translit_maybe2(
                &self.query,
                global.translit,
            ))?
        } else {
            Query::from_str(&self.query)?
        };

        let mut ftable: HashMap<Vec<String>, u64> = HashMap::new();
        let options = QueryOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100f64)
            .case_ignore(self.ignore_case);

        let writer: Box<dyn Write> = match self.output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout()),
        };

        let mut writer = csv::WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer);

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
                        let outcome = record.query(&query, &options);
                        for key in outcome.clone().into_iter() {
                            if key.iter().any(|e| !e.is_empty()) {
                                *ftable.entry(key).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(|s| s.trim()))?;
        }

        let mut ftable_sorted: Vec<(&Vec<String>, &u64)> =
            ftable.iter().collect();

        if self.reverse {
            ftable_sorted.sort_by(|a, b| match a.1.cmp(b.1) {
                Ordering::Equal => a.0.cmp(b.0),
                ordering => ordering,
            });
        } else {
            ftable_sorted.sort_by(|a, b| match b.1.cmp(a.1) {
                Ordering::Equal => a.0.cmp(b.0),
                ordering => ordering,
            });
        }

        for (i, (values, frequency)) in ftable_sorted.iter().enumerate()
        {
            if self.limit > 0 && i >= self.limit {
                break;
            }

            if **frequency < self.threshold {
                break;
            }

            let mut record = values
                .iter()
                .map(|s| translit_maybe(s, self.translit.as_deref()))
                .collect::<Vec<_>>();

            record.push(frequency.to_string());
            writer.write_record(record)?;
        }

        writer.flush()?;
        Ok(())
    }
}
