use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, Write};

use clap::Parser;
use pica_path::PathExt;
use pica_record::io::{ReaderBuilder, RecordsIterator};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::util::CliResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct HashConfig {
    /// Skip invalid records that can't be decoded.
    pub(crate) skip_invalid: Option<bool>,
}

/// Compute SHA-256 checksum of records.
#[derive(Parser, Debug)]
pub(crate) struct Hash {
    /// Skip invalid records that can't be decoded.
    #[arg(short, long)]
    skip_invalid: bool,

    /// Comma-separated list of column names.
    #[arg(long, short = 'H', default_value = "idn,sha256")]
    header: String,

    /// Write output tab-separated (TSV)
    #[arg(long, short)]
    tsv: bool,

    /// Write output to <OUTPUT> instead of stdout
    #[arg(short, long)]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Row {
    idn: Option<String>,
    hash: String,
}

impl Hash {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.hash,
            config.global
        );

        let writer: Box<dyn Write> = match self.output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout()),
        };

        let mut writer = csv::WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer);

        writer
            .write_record(self.header.split(',').map(|s| s.trim()))?;

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
                        if let Some(idn) = record.idn() {
                            let hash = record.sha256().iter().fold(
                                String::new(),
                                |mut out, b| {
                                    let _ = write!(out, "{b:02x}");
                                    out
                                },
                            );

                            writer.write_record(&[
                                idn.to_string(),
                                hash,
                            ])?;
                        }
                    }
                }
            }
        }

        writer.flush()?;
        Ok(())
    }
}
