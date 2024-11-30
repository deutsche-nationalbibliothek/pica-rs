use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{
    self, stdin, BufRead, BufReader, BufWriter, Read, Write,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use flate2::read::GzDecoder;
use pica_record::ByteRecord;

use crate::config::Config;
use crate::error::CliResult;
use crate::progress::Progress;

/// Write input lines, which can't be decoded as normalized PICA+
///
/// Read lines from files or stdin and write input lines, which can't be
/// decoded as normalized PICA+. The output is given in chronological
/// order.
#[derive(Debug, Parser)]
pub(crate) struct Invalid {
    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to FILE instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

fn reader(filename: &OsString) -> io::Result<BufReader<Box<dyn Read>>> {
    let path = PathBuf::from(filename);
    let ext = path.extension().and_then(OsStr::to_str);
    let inner: Box<dyn Read> = match ext {
        Some("gz") => Box::new(GzDecoder::new(File::open(path)?)),
        _ => {
            if path.to_str() != Some("-") {
                Box::new(File::open(path)?)
            } else {
                Box::new(stdin().lock())
            }
        }
    };

    Ok(BufReader::new(inner))
}

fn writer<P>(output: Option<P>) -> io::Result<BufWriter<Box<dyn Write>>>
where
    P: AsRef<Path>,
{
    if let Some(path) = output {
        Ok(BufWriter::new(Box::new(File::create(path)?)))
    } else {
        Ok(BufWriter::new(Box::new(io::stdout().lock())))
    }
}

impl Invalid {
    pub(crate) fn execute(self, _config: &Config) -> CliResult {
        let mut progress = Progress::new(self.progress);
        let mut writer = writer(self.output)?;
        let mut buf = Vec::<u8>::new();

        for filename in self.filenames.iter() {
            let mut reader = reader(filename)?;
            loop {
                match reader.read_until(b'\n', &mut buf)? {
                    0 => break,
                    _ => {
                        if ByteRecord::from_bytes(&buf).is_ok() {
                            progress.update(false);
                        } else {
                            writer.write_all(&buf)?;
                            progress.update(true);
                        }
                    }
                }

                buf.clear();
            }
        }

        progress.finish();
        writer.flush()?;

        Ok(ExitCode::SUCCESS)
    }
}
