use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

use bstr::ByteSlice;
use clap::Parser;
use pica_record::prelude::*;

use crate::prelude::*;
use crate::utils::FilterSet;

/// Print records in human readable format
#[derive(Parser, Debug)]
pub(crate) struct Print {
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

    #[command(flatten, next_help_heading = "Filter options")]
    filter_opts: FilterOpts,
}

impl Print {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut count = 0;

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;
        let options = MatcherOptions::from(&self.filter_opts);

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
                        progress.update(false);
                        if let Some(ref matcher) = matcher
                            && !matcher.is_match(record, &options)
                        {
                            continue;
                        }

                        if !filter_set.check(record) {
                            continue;
                        }

                        let translit = translit(self.nf.clone());

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

                        if self.filter_opts.limit > 0
                            && count >= self.filter_opts.limit
                        {
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
