use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};

use clap::Parser;
use pica::{Error, ParsePicaError, Reader, ReaderBuilder};

use crate::util::{CliError, CliResult};

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
            let builder = ReaderBuilder::new().skip_invalid(false);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for result in reader.records() {
                match result {
                    Err(Error::InvalidRecord(ParsePicaError {
                        data,
                        ..
                    })) => {
                        writer.write_all(&data)?;
                    }
                    Err(e) => return Err(CliError::from(e)),
                    _ => continue,
                }
            }
        }

        writer.flush()?;
        Ok(())
    }
}
