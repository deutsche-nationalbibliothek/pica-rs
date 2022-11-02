use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;
use pica_record::io::{BufReadExt, WriterBuilder};
use pica_record::ParsePicaError;
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

/// Concatenate records from multiple files
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

        for filename in self.filenames {
            let mut reader = config.reader(filename)?;

            reader.for_pica_record(|result| match result {
                Err(ParsePicaError::InvalidRecord(_))
                    if skip_invalid =>
                {
                    Ok(true)
                }
                Err(e) => Err(e.into()),
                Ok(record) => {
                    writer.write_byte_record(&record)?;
                    if let Some(ref mut writer) = tee_writer {
                        writer.write_byte_record(&record)?;
                    }

                    Ok(true)
                }
            })?;
        }

        writer.finish()?;
        if let Some(ref mut writer) = tee_writer {
            writer.finish()?;
        }

        Ok(())
    }
}
