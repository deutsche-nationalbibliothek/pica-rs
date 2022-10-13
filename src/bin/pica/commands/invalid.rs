use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

use bstr::io::BufReadExt;
use clap::Parser;
use flate2::read::GzDecoder;
use pica_record::{ByteRecord, ParsePicaError};

use crate::util::CliResult;

#[derive(Parser, Debug)]
pub(crate) struct Invalid {
    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Invalid {
    pub(crate) fn run(self) -> CliResult<()> {
        let writer: Box<dyn Write> = match self.output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout()),
        };

        let mut writer = BufWriter::new(writer);

        for filename in self.filenames {
            let path = Path::new(&filename);
            let reader: Box<dyn Read> =
                match path.extension().and_then(OsStr::to_str) {
                    Some("gz") => {
                        Box::new(GzDecoder::new(File::open(filename)?))
                    }
                    _ => {
                        if filename != "-" {
                            Box::new(File::open(filename)?)
                        } else {
                            Box::new(io::stdin())
                        }
                    }
                };

            let mut reader = BufReader::new(reader);
            reader.for_byte_line_with_terminator(|line| {
                match ByteRecord::from_bytes(line) {
                    Err(ParsePicaError::InvalidRecord(data)) => {
                        writer.write_all(&data)?;
                        Ok(true)
                    }
                    _ => Ok(true),
                }
            })?;
        }

        writer.flush()?;
        Ok(())
    }
}
