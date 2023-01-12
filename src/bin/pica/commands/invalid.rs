use std::ffi::OsString;
use std::io::Write;

use clap::Parser;
use pica_record::io::{ReadPicaError, ReaderBuilder, RecordsIterator};
use pica_record::ParsePicaError;

use crate::config::Config;
use crate::util::CliResult;

/// Filter out invalid records, which can't be decoded
///
/// Read lines from files or stdin and filter out invalid records,
/// which can't be decoded as normalized PICA+. The output is given in
/// chronological order.
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
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let mut writer = config.writer(self.output)?;

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                match result {
                    Err(ReadPicaError::Parse(
                        ParsePicaError::InvalidRecord(data),
                    )) => {
                        writer.write_all(&data)?;
                    }
                    Err(e) => return Err(e.into()),
                    _ => continue,
                }
            }
        }

        writer.flush()?;
        Ok(())
    }
}
