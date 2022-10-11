use std::ffi::OsString;
use std::io::{self, Read};
use std::path::PathBuf;

use clap::Parser;
use pica::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::CliResult;
use crate::{gzip_flag, skip_invalid_flag};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CatConfig {
    /// Skip invalid records that can't be decoded.
    pub(crate) skip_invalid: Option<bool>,

    /// Compress output in gzip format
    pub(crate) gzip: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Cat {
    /// Skip invalid records that can't be decoded.
    #[arg(short, long)]
    skip_invalid: bool,

    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// Write simultaneously to the file <TEE> and stdout
    #[arg(long)]
    tee: Option<PathBuf>,

    /// Compress output in gzip format
    #[arg(short, long, requires = "output")]
    gzip: bool,

    /// Write output to <OUTPUT> instead of stdout
    #[arg(short, long)]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-")]
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

        let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
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

                writer.write_byte_record(&record)?;

                if let Some(ref mut writer) = tee_writer {
                    writer.write_byte_record(&record)?;
                }
            }
        }

        writer.finish()?;
        if let Some(ref mut writer) = tee_writer {
            writer.finish()?;
        }

        Ok(())
    }
}
