use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, value_parser};
use pica_record::prelude::*;

use crate::config::Config;
use crate::error::CliResult;
use crate::progress::Progress;

/// Splits a list of records into chunks
///
/// This command is used to split a list of records into chunks of a
/// given size. To write all chunks in a directory, use the `--outdir`
/// or `-o` option (if the directory doesn't exist, the directory will
/// be created automatically).
#[derive(Parser, Debug)]
pub(crate) struct Split {
    /// Skip invalid records that can't be decoded as normalized PICA+
    #[arg(short, long)]
    skip_invalid: bool,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long)]
    progress: bool,

    /// Write partitions into OUTDIR
    #[arg(long, short, value_name = "outdir", default_value = ".")]
    outdir: PathBuf,

    /// Filename template ("{}" is replaced by the chunk number)
    #[arg(long, value_name = "template")]
    template: Option<String>,

    /// Chunk size
    #[arg(value_parser = value_parser!(u32).range(1..))]
    chunk_size: u32,

    /// Read one or more files in normalized PICA+ format
    ///
    /// If no filenames where given or a filename is "-", data is read
    /// from standard input (stdin).
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Split {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut chunks: u32 = 0;
        let mut count = 0;

        let template = self.template.unwrap_or(if self.gzip {
            "{}.dat.gz".into()
        } else {
            "{}.dat".into()
        });

        if !self.outdir.exists() {
            fs::create_dir_all(&self.outdir)?;
        }

        let mut writer = WriterBuilder::new()
            .gzip(self.gzip)
            .from_path(
                self.outdir
                    .join(template.replace("{}", &chunks.to_string())),
            )?;

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

                        if count > 0
                            && count as u32 % self.chunk_size == 0
                        {
                            writer.finish()?;
                            chunks += 1;

                            writer = WriterBuilder::new()
                                .gzip(self.gzip)
                                .from_path(self.outdir.join(
                                    template.replace(
                                        "{}",
                                        &chunks.to_string(),
                                    ),
                                ))?;
                        }

                        writer.write_byte_record(record)?;
                        count += 1;
                    }
                }
            }
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
