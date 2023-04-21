use std::ffi::OsString;

use clap::{Parser, ValueEnum};
use pica_record::io::{
    ByteRecordWrite, ReaderBuilder, RecordsIterator,
};
use serde::{Deserialize, Serialize};

use self::plain::PlainWriter;
use self::xml::XmlWriter;
use crate::util::CliError;
use crate::{skip_invalid_flag, CliResult, Config};

mod plain;
mod xml;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ConvertConfig {
    /// Skip invalid records that can't be decoded.
    pub(crate) skip_invalid: Option<bool>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    Plus,
    Plain,
    Xml,
}

/// Convert PICA+ into other formats.
#[derive(Parser, Debug)]
pub(crate) struct Convert {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// The input format. Currently only PICA+ is supported.
    #[arg(
        short,
        long,
        value_enum,
        default_value = "plus",
        hide_default_value = true,
        value_name = "FORMAT"
    )]
    from: Format,

    /// The output format.
    #[arg(
        short,
        long,
        value_enum,
        default_value = "plus",
        hide_default_value = true,
        value_name = "FORMAT"
    )]
    to: Format,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Convert {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.convert,
            config.global
        );

        let mut writer: Box<dyn ByteRecordWrite> =
            match (self.from, self.to) {
                (_, Format::Plain) => {
                    Box::new(PlainWriter::new(self.output)?)
                }
                (_, Format::Xml) => {
                    Box::new(XmlWriter::new(self.output)?)
                }
                (from, to) => {
                    return Err(CliError::Other(format!(
                        "convert from {:?} to {:?} is not supported",
                        from, to,
                    )));
                }
            };

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
                        writer.write_byte_record(&record)?;
                    }
                }
            }
        }

        writer.finish()?;
        Ok(())
    }
}
