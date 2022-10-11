use std::ffi::OsString;
use std::io::{self, Read};

use clap::Parser;
use pica::{PicaWriter, Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::{CliError, CliResult};
use crate::{gzip_flag, skip_invalid_flag};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SliceConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Slice {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// The lower bound of the range (inclusive)
    #[arg(long, default_value = "0")]
    start: usize,
    /// The upper bound of the range (exclusive)
    #[arg(long, default_value = "0")]
    end: usize,

    /// The length of the slice
    #[arg(long, default_value = "0", conflicts_with = "end")]
    length: usize,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Slice {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let gzip_compression = gzip_flag!(self.gzip, config.slice);
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.slice,
            config.global
        );

        let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
            .gzip(gzip_compression)
            .from_path_or_stdout(self.output)?;

        let mut range = if self.end > 0 {
            self.start..self.end
        } else if self.length > 0 {
            self.start..(self.start + self.length)
        } else {
            self.start..::std::usize::MAX
        };

        let mut i = 0;

        for filename in self.filenames {
            let builder = ReaderBuilder::new().skip_invalid(false);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for result in reader.byte_records() {
                match result {
                    Ok(record) => {
                        if range.contains(&i) {
                            writer.write_byte_record(&record)?;
                        } else if i < range.start {
                            i += 1;
                            continue;
                        } else {
                            break;
                        }
                    }
                    Err(e) if !skip_invalid => {
                        return Err(CliError::from(e))
                    }
                    _ => {
                        if self.length > 0
                            && range.end < std::usize::MAX
                        {
                            range.end += 1;
                        }
                    }
                }

                i += 1;
            }
        }

        writer.finish()?;
        Ok(())
    }
}
