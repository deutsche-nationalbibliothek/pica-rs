use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::ExitCode;

use clap::{value_parser, Parser};
use pica_record::prelude::*;

use crate::config::Config;
use crate::error::CliResult;
use crate::prelude::translit;
use crate::progress::Progress;

/// Count records, fields and subfields
#[derive(Parser, Debug)]
pub(crate) struct Count {
    /// Whether to skip invalid records or not
    #[arg(short, long)]
    skip_invalid: bool,

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

    /// When this flag is set, comparison operations will be search
    /// case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons (0 <= score
    /// < 100).
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
          default_value = "75")]
    strsim_threshold: u8,

    /// A filter expression used for searching
    #[arg(long = "where", value_name = "FILTER")]
    filter: Option<String>,

    /// Connects the where clause with additional expressions using the
    /// logical AND-operator (conjunction)
    ///
    /// This option can't be combined with `--or`.
    #[arg(long, requires = "filter", conflicts_with = "or")]
    and: Vec<String>,

    /// Connects the where clause with additional expressions using the
    /// logical OR-operator (disjunction)
    ///
    /// This option can't be combined with `--and` or `--not`.
    #[arg(long, requires = "filter", conflicts_with_all = ["and", "not"])]
    or: Vec<String>,

    /// Connects the where clause with additional expressions using the
    /// logical NOT-operator (negation)
    ///
    /// This option can't be combined with `--and` or `--or`.
    #[arg(long, requires = "filter", conflicts_with = "or")]
    not: Vec<String>,

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
}

impl Count {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let translit = translit(config.normalization.as_ref());

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

        let matcher = if let Some(matcher) = self.filter {
            Some(
                RecordMatcherBuilder::with_transform(
                    matcher, translit,
                )?
                .and(self.and)?
                .or(self.or)?
                .not(self.not)?
                .build(),
            )
        } else {
            None
        };

        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100f64)
            .case_ignore(self.ignore_case);

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
                        if let Some(ref matcher) = matcher {
                            if !matcher.is_match(&record, &options) {
                                continue;
                            }
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
