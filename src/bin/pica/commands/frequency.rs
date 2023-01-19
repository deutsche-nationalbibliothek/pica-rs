use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::str::FromStr;

use bstr::BString;
use clap::Parser;
use pica_path::{Path, PathExt};
use pica_record::io::{ReaderBuilder, RecordsIterator};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
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
/// euqal, these lines are given in lexicographical order.
#[derive(Parser, Debug)]
pub(crate) struct Frequency {
    /// Skip invalid records that can't be decoded as normalized PICA+.
    #[arg(long, short)]
    skip_invalid: bool,

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

    /// Ignore rows with a frequency ≤ <t>.
    #[arg(
        long,
        short,
        value_name = "t",
        default_value = "0",
        hide_default_value = true
    )]
    threshold: u64,

    /// Comma-separated list of column names.
    #[arg(long, short = 'H')]
    header: Option<String>,

    /// Transliterate output into the selected normalform <NF>
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
    path: String,

    /// Read one or more files in normalized PICA+ format. With no
    /// files, or when a filename is '-', read from stanard input
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

        let mut ftable: HashMap<BString, u64> = HashMap::new();
        let path = Path::from_str(&self.path)?;

        let writer: Box<dyn Write> = match self.output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout()),
        };

        let mut writer = csv::WriterBuilder::new().from_writer(writer);

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
                        for value in record.path(&path) {
                            *ftable
                                .entry(BString::from(value.to_vec()))
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(|s| s.trim()))?;
        }

        let mut ftable_sorted: Vec<(&BString, &u64)> =
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

        for (i, (value, frequency)) in ftable_sorted.iter().enumerate()
        {
            if self.limit > 0 && i >= self.limit {
                break;
            }

            if **frequency <= self.threshold {
                break;
            }

            let value = translit_maybe(
                &value.to_string(),
                self.translit.as_deref(),
            );

            writer.write_record(&[value, frequency.to_string()])?;
        }

        writer.flush()?;
        Ok(())
    }
}
