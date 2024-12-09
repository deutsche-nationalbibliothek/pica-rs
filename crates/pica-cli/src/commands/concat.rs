use std::ffi::OsString;
use std::fmt::Write;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use hashbrown::HashSet;
use pica_record::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::CliResult;
use crate::progress::Progress;

/// Concatenate records from multiple inputs
#[derive(Debug, Parser)]
#[clap(visible_alias = "cat")]
pub(crate) struct Concat {
    /// Whether to skip invalid records or not
    #[arg(short, long)]
    skip_invalid: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write simultaneously to the file <TEE> and stdout
    #[arg(long)]
    tee: Option<OsString>,

    /// Whether to skip duplicate records or not.
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
    /// written to OUTPUT.
    #[arg(
        long,
        requires = "unique",
        value_name = "STRATEGY",
        hide_possible_values = true,
        default_value = "idn"
    )]
    unique_strategy: Strategy,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// Write output to OUTPUT instead of stdout
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<OsString>,
}

#[derive(
    Clone, Debug, PartialEq, Default, ValueEnum, Deserialize, Serialize,
)]
pub(crate) enum Strategy {
    #[default]
    #[serde(rename = "idn")]
    Idn,
    #[serde(rename = "hash")]
    Hash,
}

#[inline]
fn record_key(record: &ByteRecord, strategy: &Strategy) -> String {
    match strategy {
        Strategy::Idn => record.ppn().to_string(),
        Strategy::Hash => {
            record.sha256().iter().fold(String::new(), |mut out, b| {
                let _ = write!(out, "{b:02x}");
                out
            })
        }
    }
}

impl Concat {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut seen = HashSet::new();

        let mut writer = WriterBuilder::new()
            .append(self.append)
            .from_path_or_stdout(self.output)?;

        let mut tee_writer = match self.tee {
            Some(path) => Some(
                WriterBuilder::new()
                    .append(self.append)
                    .from_path(path)?,
            ),
            None => None,
        };

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next_byte_record() {
                match result {
                    Err(e) if e.skip_parse_err(skip_invalid) => {
                        progress.update(true);
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                    Ok(ref record) => {
                        if self.unique {
                            let key = record_key(
                                record,
                                &self.unique_strategy,
                            );
                            if seen.contains(&key) || key.is_empty() {
                                continue;
                            }

                            seen.insert(key);
                        }

                        writer.write_byte_record(record)?;
                        progress.update(false);

                        if let Some(ref mut wtr) = tee_writer {
                            wtr.write_byte_record(record)?;
                        }
                    }
                }
            }
        }

        if let Some(ref mut wtr) = tee_writer {
            wtr.finish()?;
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
