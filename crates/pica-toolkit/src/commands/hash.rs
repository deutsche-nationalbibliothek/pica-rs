use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, Write};

use clap::Parser;
use pica_path::PathExt;
use pica_record_v1::io::{ReaderBuilder, RecordsIterator};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::CliResult;
use crate::progress::Progress;
use crate::skip_invalid_flag;

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

impl Hash {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let mut progress = Progress::new(self.progress);
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

        writer.write_record(self.header.split(',').map(str::trim))?;

        for filename in self.filenames {
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

                let record = result.unwrap();
                progress.record();

                if let Some(idn) = record.idn() {
                    let hash = record.sha256().iter().fold(
                        String::new(),
                        |mut out, b| {
                            let _ = write!(out, "{b:02x}");
                            out
                        },
                    );

                    writer.write_record(&[idn.to_string(), hash])?;
                }
            }
        }

        progress.finish();
        writer.flush()?;
        Ok(())
    }
}
