use std::ffi::OsString;
use std::fs::create_dir;
use std::path::PathBuf;

use clap::{value_parser, Parser};
use pica_record::io::{ReaderBuilder, RecordsIterator, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::progress::Progress;
use crate::util::CliResult;
use crate::{gzip_flag, skip_invalid_flag, template_opt};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SplitConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
    pub(crate) template: Option<String>,
}

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

    /// Write partitions into <outdir>
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
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let gzip_compression = gzip_flag!(self.gzip, config.split);
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.split,
            config.global
        );

        let filename_template = template_opt!(
            self.template,
            config.split,
            if gzip_compression {
                "{}.dat.gz"
            } else {
                "{}.dat"
            }
        );

        if !self.outdir.exists() {
            create_dir(&self.outdir)?;
        }

        let mut chunks: u32 = 0;
        let mut count = 0;
        let mut progress = Progress::new(self.progress);
        let mut writer =
            WriterBuilder::new().gzip(gzip_compression).from_path(
                self.outdir
                    .join(
                        filename_template
                            .replace("{}", &chunks.to_string()),
                    )
                    .to_str()
                    .unwrap(),
            )?;

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

                        if count > 0
                            && count as u32 % self.chunk_size == 0
                        {
                            writer.finish()?;
                            chunks += 1;

                            writer = WriterBuilder::new()
                                .gzip(gzip_compression)
                                .from_path(
                                    self.outdir
                                        .join(
                                            filename_template.replace(
                                                "{}",
                                                &chunks.to_string(),
                                            ),
                                        )
                                        .to_str()
                                        .unwrap(),
                                )?;
                        }

                        writer.write_byte_record(&record)?;
                        count += 1;
                    }
                }
            }
        }

        progress.finish();
        writer.finish()?;

        Ok(())
    }
}
