use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::str::FromStr;

use bstr::BString;
use clap::Parser;
use pica::{Path, Reader, ReaderBuilder};
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

#[derive(Parser, Debug)]
pub(crate) struct Frequency {
    /// Skip invalid records that can't be decoded
    #[arg(long, short)]
    skip_invalid: bool,

    /// Sort results in reverse order
    #[arg(long, short)]
    reverse: bool,

    /// Limit result to the <n> most common values
    #[arg(
        long,
        short,
        value_name = "n",
        hide_default_value = true,
        default_value = "0"
    )]
    limit: usize,

    /// Ignore rows with a frequency ≤ <t>
    #[arg(
        long,
        short,
        value_name = "t",
        default_value = "0",
        hide_default_value = true
    )]
    threshold: u64,

    /// Comma-separated list of column names
    #[arg(long, short = 'H')]
    header: Option<String>,

    /// Transliterate output into the selected normalform <NF>
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc")
    #[arg(long,
          value_name = "NF", 
          value_parser = ["nfd", "nfkd", "nfc", "nfkc"],
          hide_possible_values = true,
    )]
    translit: Option<String>,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// A PICA path expression
    path: String,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-")]
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

                for value in record.path(&path) {
                    *ftable.entry(value.to_owned()).or_insert(0) += 1;
                }
            }
        }

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(|s| s.trim()))?;
        }

        let mut ftable_sorted: Vec<(&BString, &u64)> =
            ftable.iter().collect();
        if self.reverse {
            ftable_sorted.sort_by(|a, b| a.1.cmp(b.1));
        } else {
            ftable_sorted.sort_by(|a, b| b.1.cmp(a.1));
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
