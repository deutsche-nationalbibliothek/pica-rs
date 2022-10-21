use std::ffi::OsString;
use std::io::Write;

use clap::Parser;
use pica_record::io::BufReadExt;
use pica_record::ParsePicaError;

use crate::config::Config;
use crate::util::CliResult;

/// Filter out invalid records, which can't be decoded
///
/// Read lines from files or stdin and filter out invalid records,
/// which can't be decoded as normalized PICA+. The output is given
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
            let mut reader = config.reader(filename)?;

            reader.for_pica_record(|result| match result {
                Err(ParsePicaError::InvalidRecord(data)) => {
                    writer.write_all(&data)?;
                    Ok(true)
                }
                Err(e) => Err(e.into()),
                _ => Ok(true),
            })?;
        }

        writer.flush()?;
        Ok(())
    }
}
