mod binary;
mod import;
mod json;
mod plain;
mod xml;

use std::ffi::OsString;

use clap::{Parser, ValueEnum};
use pica_record::io::{
    ByteRecordWrite, ReaderBuilder, RecordsIterator, WriterBuilder,
};
use serde::{Deserialize, Serialize};

use self::binary::BinaryWriter;
use self::import::ImportWriter;
use self::json::JsonWriter;
use self::plain::PlainWriter;
use self::xml::XmlWriter;
use crate::progress::Progress;
use crate::util::CliError;
use crate::{skip_invalid_flag, CliResult, Config};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ConvertConfig {
    /// Skip invalid records that can't be decoded.
    pub(crate) skip_invalid: Option<bool>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    Binary,
    Import,
    Json,
    Plain,
    Plus,
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

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

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

        if self.from != Format::Plus {
            return Err(CliError::Other(format!(
                "convert from {:?} is not supported",
                self.from
            )));
        }

        let mut writer: Box<dyn ByteRecordWrite> = match self.to {
            Format::Plus => {
                WriterBuilder::new().from_path_or_stdout(self.output)?
            }
            Format::Binary => Box::new(BinaryWriter::new(self.output)?),
            Format::Import => Box::new(ImportWriter::new(self.output)?),
            Format::Json => Box::new(JsonWriter::new(self.output)?),
            Format::Plain => Box::new(PlainWriter::new(self.output)?),
            Format::Xml => Box::new(XmlWriter::new(self.output)?),
        };

        let mut progress = Progress::new(self.progress);

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
                        writer.write_byte_record(&record)?;
                    }
                }
            }
        }

        progress.finish();
        writer.finish()?;

        Ok(())
    }
}
