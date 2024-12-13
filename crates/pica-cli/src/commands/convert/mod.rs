use std::ffi::OsString;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use pica_record::prelude::*;

use self::binary::BinaryWriter;
use self::import::ImportWriter;
use self::json::JsonWriter;
use self::plain::PlainWriter;
use self::xml::XmlWriter;
use crate::prelude::*;

mod binary;
mod import;
mod json;
mod plain;
mod xml;

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

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Convert {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);

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
                        writer.write_byte_record(record)?;
                        progress.update(false);
                    }
                }
            }
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
