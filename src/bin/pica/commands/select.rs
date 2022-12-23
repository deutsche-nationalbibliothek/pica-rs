use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};

use clap::Parser;
use pica::matcher::{MatcherFlags, RecordMatcher};
use pica::{Outcome, Reader, ReaderBuilder, Selectors};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::{CliError, CliResult};

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

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Comma-separated list of selectors
    selectors: String,

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
        let mut writer = csv::WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer(self.output, self.append)?);

        let selectors = match Selectors::decode(&self.selectors) {
            Ok(val) => val,
            _ => {
                return Err(CliError::Other(format!(
                    "invalid select list: {}",
                    self.selectors
                )))
            }
        };

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(|s| s.trim()))?;
        }

        let flags = MatcherFlags::default();
        let filter = match self.filter {
            Some(filter_str) => match RecordMatcher::new(&filter_str) {
                Ok(f) => f,
                _ => {
                    return Err(CliError::Other(format!(
                        "invalid filter: \"{filter_str}\""
                    )))
                }
            },
            None => RecordMatcher::True,
        };

        for filename in self.filenames {
            let builder =
                ReaderBuilder::new().skip_invalid(skip_invalid);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for result in reader.records() {
                let record = result?;

                if !filter.is_match(&record, &flags) {
                    continue;
                }

                let outcome = selectors
                    .iter()
                    .map(|selector| {
                        record.select(selector, self.ignore_case)
                    })
                    .fold(Outcome::default(), |acc, x| acc * x);

                for row in outcome.iter() {
                    if self.no_empty_columns
                        && row.iter().any(|column| column.is_empty())
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
                                row.iter()
                                    .map(ToString::to_string)
                                    .map(|s| {
                                        translit_maybe(
                                            &s,
                                            self.translit.as_deref(),
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

        writer.flush()?;
        Ok(())
    }
}
