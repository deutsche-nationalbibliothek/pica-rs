use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, value_parser};
use hashbrown::hash_map::{Entry, HashMap};
use pica_record::prelude::*;

use crate::config::Config;
use crate::error::CliResult;
use crate::progress::Progress;

/// Partition records by subfield values
///
/// The files are written to the OUTDIR directory with filenames based
/// on the values of the subfield, which is referenced by the path
/// expression.
///
/// If a record doesn't have the field/subfield, the record won't be
/// written to a partition. A record with multiple values will be
/// written to each partition; thus the partitions may not be disjoint.
/// In order to prevent duplicate records in a partition , all duplicate
/// values of a record will be removed.
#[derive(Parser, Debug)]
pub(crate) struct Partition {
    /// Whether to skip invalid records or not.
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

    /// Compress each partition in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Show progress bar
    #[arg(short, long)]
    progress: bool,

    /// Write partitions into OUTDIR
    ///
    /// If the directory doesn't exists, it will be created
    /// automatically.
    #[arg(long, short, value_name = "OUTDIR", default_value = ".")]
    outdir: PathBuf,

    /// Filename template ("{}" is replaced by subfield value)
    #[arg(long, short, value_name = "template")]
    template: Option<String>,

    /// A path expression (e.g. "002@.0")
    path: Path,

    /// Read one or more files in normalized PICA+ format
    ///
    /// If no filenames where given or a filename is "-", data is read
    /// from standard input (stdin).
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Partition {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let template = self.template.unwrap_or(if self.gzip {
            "{}.dat.gz".into()
        } else {
            "{}.dat".into()
        });

        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100f64)
            .case_ignore(self.ignore_case);

        if !self.outdir.exists() {
            fs::create_dir_all(&self.outdir)?;
        }

        let mut writers: HashMap<Vec<u8>, Box<dyn ByteRecordWrite>> =
            HashMap::new();

        let mut progress = Progress::new(self.progress);

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
                        progress.update(false);

                        let mut values: Vec<_> =
                            record.path(&self.path, &options).collect();
                        values.sort_unstable();
                        values.dedup();

                        for value in values {
                            let mut entry =
                                writers.entry(value.to_vec());
                            let writer = match entry {
                                Entry::Vacant(vacant) => {
                                    let filename = template.replace(
                                        "{}",
                                        &value.to_string(),
                                    );

                                    let path =
                                        self.outdir.join(filename);
                                    let writer = WriterBuilder::new()
                                        .gzip(self.gzip)
                                        .from_path(path)?;

                                    vacant.insert(writer)
                                }
                                Entry::Occupied(ref mut occupied) => {
                                    occupied.get_mut()
                                }
                            };

                            writer.write_byte_record(record)?;
                        }
                    }
                }
            }
        }

        progress.finish();
        for (_, mut writer) in writers {
            writer.finish()?;
        }

        Ok(ExitCode::SUCCESS)
    }
}
