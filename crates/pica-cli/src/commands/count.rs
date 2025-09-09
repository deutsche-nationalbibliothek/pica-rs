use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;
use pica_record::prelude::*;

use crate::prelude::*;

/// Count records, fields and subfields
#[derive(Parser, Debug)]
pub(crate) struct Count {
    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// Prints only the number of records
    #[arg(long, short,
          conflicts_with_all = ["fields", "subfields", "csv", "tsv", "no_header"])]
    records: bool,

    /// Prints only the number of fields
    #[arg(long,
          conflicts_with_all = ["records", "subfields", "csv", "tsv", "no_header"])]
    fields: bool,

    /// Prints only the number of subfields
    #[arg(long,
          conflicts_with_all = ["records", "fields", "csv", "tsv", "no_header"])]
    subfields: bool,

    /// Write output comma-separated (CSV)
    #[arg(long, conflicts_with = "tsv")]
    csv: bool,

    /// Write output tab-separated (TSV)
    #[arg(long)]
    tsv: bool,

    /// Do not write header row
    #[arg(long)]
    no_header: bool,

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
    pub(crate) filter_opts: FilterOpts,
}

impl Count {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);

        let mut writer: Box<dyn Write> = match self.output {
            Some(path) => Box::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(!self.append)
                    .append(self.append)
                    .open(path)?,
            ),
            None => Box::new(io::stdout().lock()),
        };

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let options = MatcherOptions::from(&self.filter_opts);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;

        let mut records = 0;
        let mut fields = 0;
        let mut subfields = 0;

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
                    Ok(record) => {
                        progress.update(false);

                        if !filter_set.check(&record) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher
                            && !matcher.is_match(&record, &options)
                        {
                            continue;
                        }

                        records += 1;
                        fields += record.fields().len();
                        subfields += record
                            .fields()
                            .iter()
                            .map(|field| field.subfields().len())
                            .sum::<usize>();
                    }
                }
            }
        }

        if self.records {
            writeln!(writer, "{records}")?;
        } else if self.fields {
            writeln!(writer, "{fields}")?;
        } else if self.subfields {
            writeln!(writer, "{subfields}")?;
        } else if self.csv {
            if !self.no_header {
                writeln!(writer, "records,fields,subfields")?;
            }
            writeln!(writer, "{records},{fields},{subfields}")?;
        } else if self.tsv {
            if !self.no_header {
                writeln!(writer, "records\tfields\tsubfields")?;
            }
            writeln!(writer, "{records}\t{fields}\t{subfields}")?;
        } else {
            writeln!(writer, "records: {records}")?;
            writeln!(writer, "fields: {fields}")?;
            writeln!(writer, "subfields: {subfields}")?;
        }

        progress.finish();
        writer.flush()?;

        Ok(ExitCode::SUCCESS)
    }
}
