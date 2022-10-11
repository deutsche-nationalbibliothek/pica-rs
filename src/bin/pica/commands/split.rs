use std::ffi::OsString;
use std::fs::create_dir;
use std::io::{self, Read};
use std::path::PathBuf;

use clap::{value_parser, Parser};
use pica::{Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::CliResult;
use crate::{gzip_flag, skip_invalid_flag, template_opt};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SplitConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
    pub(crate) template: Option<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct Split {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Write partitions into <outdir> (default value ".")
    #[arg(long, short, value_name = "outdir", default_value = ".")]
    outdir: PathBuf,

    /// Filename template ("{}" is replaced by subfield value)
    #[arg(long, short, value_name = "template")]
    template: Option<String>,

    /// Split size
    #[arg(default_value = "500", 
          value_parser = value_parser!(u32).range(1..))]
    chunk_size: u32,

    /// Read one or more files in normalized PICA+ format.
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
            let builder =
                ReaderBuilder::new().skip_invalid(skip_invalid);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for result in reader.byte_records() {
                let record = result?;

                if count > 0 && count as u32 % self.chunk_size == 0 {
                    writer.finish()?;
                    chunks += 1;

                    writer =
                        WriterBuilder::new()
                            .gzip(gzip_compression)
                            .from_path(
                                self.outdir
                                    .join(filename_template.replace(
                                        "{}",
                                        &chunks.to_string(),
                                    ))
                                    .to_str()
                                    .unwrap(),
                            )?;
                }

                writer.write_byte_record(&record)?;
                count += 1;
            }
        }

        writer.finish()?;
        Ok(())
    }
}
