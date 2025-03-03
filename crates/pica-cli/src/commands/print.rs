use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

use bstr::ByteSlice;
use clap::Parser;
use pica_record::prelude::*;

use crate::prelude::*;

/// Print records in human readable format
#[derive(Parser, Debug)]
pub(crate) struct Print {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Limit the result to first N records
    #[arg(long, short, value_name = "N", default_value = "0")]
    limit: usize,

    /// Transliterate output into the selected normal form NF
    #[arg(long = "translit", value_name = "NF")]
    nf: Option<NormalizationForm>,

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

impl Print {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut count = 0;

        let mut writer: BufWriter<Box<dyn Write>> =
            if let Some(path) = self.output {
                BufWriter::new(Box::new(File::create(path)?))
            } else {
                BufWriter::new(Box::new(io::stdout().lock()))
            };

        'outer: for filename in self.filenames {
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
                        let translit = translit(self.nf.clone());
                        progress.update(false);

                        for field in record.fields() {
                            field.tag().write_to(&mut writer)?;
                            if let Some(occ) = field.occurrence() {
                                occ.write_to(&mut writer)?;
                            }

                            for subfield in field.subfields() {
                                let code = subfield.code();
                                write!(writer, " ${code}")?;

                                let value = translit(
                                    subfield.value().to_str().unwrap(),
                                );
                                write!(writer, " {value}")?;
                            }

                            writeln!(writer)?;
                        }

                        writeln!(writer)?;
                        count += 1;

                        if self.limit > 0 && count >= self.limit {
                            break 'outer;
                        }
                    }
                }
            }
        }

        progress.finish();
        writer.flush()?;

        Ok(ExitCode::SUCCESS)
    }
}
