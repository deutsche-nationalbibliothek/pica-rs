use std::ffi::OsString;
use std::io::Write;

use clap::Parser;
use pica_record::io::{ReadPicaError, ReaderBuilder, RecordsIterator};
use pica_record::ParsePicaError;

use crate::config::Config;
use crate::progress::Progress;
use crate::util::CliResult;

/// Write input lines, which can't be decoded as normalized PICA+
///
/// Read lines from files or stdin and write input lines, which can't be
/// decoded as normalized PICA+. The output is given in chronological
/// order.
#[derive(Parser, Debug)]
pub(crate) struct Invalid {
    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

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
        let mut progress = Progress::new(self.progress);

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                match result {
                    Err(ReadPicaError::Parse {
                        msg: _,
                        err: ParsePicaError::InvalidRecord(data),
                    }) => {
                        progress.invalid();
                        writer.write_all(&data)?;
                    }
                    Err(e) => return Err(e.into()),
                    _ => {
                        progress.record();
                        continue;
                    }
                }
            }
        }

        progress.finish();
        writer.flush()?;

        Ok(())
    }
}
