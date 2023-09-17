use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt::Write;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use pica_path::PathExt;
use pica_record::io::{ReaderBuilder, RecordsIterator, WriterBuilder};
use pica_record::ByteRecord;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::progress::Progress;
use crate::util::CliResult;
use crate::{gzip_flag, skip_invalid_flag};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CatConfig {
    /// Skip invalid records that can't be decoded.
    pub(crate) skip_invalid: Option<bool>,

    /// Strategy to determine duplicate records.
    pub(crate) unique_strategy: Option<Strategy>,

    /// Compress output in gzip format
    pub(crate) gzip: Option<bool>,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Default,
    ValueEnum,
    Deserialize,
    Serialize,
)]
pub(crate) enum Strategy {
    #[default]
    #[serde(rename = "idn")]
    Idn,
    #[serde(rename = "hash")]
    Hash,
}

/// Concatenate records from multiple files
#[derive(Parser, Debug)]
pub(crate) struct Cat {
    /// Skip invalid records that can't be decoded.
    #[arg(short, long)]
    skip_invalid: bool,

    /// Skip duplicate records
    #[arg(long, short)]
    unique: bool,

    /// Use the given strategy to determine duplicate records.
    ///
    /// The `idn` strategy (default) is used to distinguish records by
    /// IDN (first value of field `003@.0`) and `hash` compares
    /// the SHA-256 checksums over all fields of a record.
    ///
    /// Note: If a record doesn't contain a IDN value and the `idn`
    /// strategy  is selected, the record is ignored and won't be
    /// written to <OUTPUT>.
    #[arg(
        long,
        requires = "unique",
        value_name = "strategy",
        hide_possible_values = true
    )]
    unique_strategy: Option<Strategy>,

    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// Write simultaneously to the file <TEE> and stdout
    #[arg(long)]
    tee: Option<PathBuf>,

    /// Compress output in gzip format
    #[arg(short, long, requires = "output")]
    gzip: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to <OUTPUT> instead of stdout
    #[arg(short, long)]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Cat {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let gzip_compression = gzip_flag!(self.gzip, config.cat);
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.cat,
            config.global
        );

        let unique_strategy =
            if let Some(strategy) = self.unique_strategy {
                strategy
            } else if let Some(ref config) = config.cat {
                config.unique_strategy.clone().unwrap_or_default()
            } else {
                Strategy::default()
            };

        let mut seen = BTreeSet::new();
        let key = |record: &ByteRecord| -> String {
            match unique_strategy {
                Strategy::Idn => record
                    .idn()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                Strategy::Hash => record.sha256().iter().fold(
                    String::new(),
                    |mut out, b| {
                        let _ = write!(out, "{b:02x}");
                        out
                    },
                ),
            }
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

        let mut progress = Progress::new(self.progress);

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                match result {
                    Err(e) => {
                        if e.is_invalid_record() && skip_invalid {
                            progress.invalid();
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                    Ok(record) => {
                        progress.record();

                        if self.unique {
                            let k = key(&record);

                            if k.is_empty() || seen.contains(&k) {
                                continue;
                            }

                            seen.insert(k);
                        }

                        writer.write_byte_record(&record)?;
                        if let Some(ref mut writer) = tee_writer {
                            writer.write_byte_record(&record)?;
                        }
                    }
                }
            }
        }

        progress.finish();
        writer.finish()?;
        if let Some(ref mut writer) = tee_writer {
            writer.finish()?;
        }

        Ok(())
    }
}
