use std::ffi::OsString;
use std::io;

use clap::{value_parser, Parser};
use pica_matcher::{MatcherOptions, RecordMatcher};
use pica_record::io::{
    ByteRecordWrite, ReaderBuilder, RecordsIterator, WriterBuilder,
};
use pica_record::{ByteRecord, Level};
use pica_utils::NormalizationForm;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::CliResult;
use crate::progress::Progress;
use crate::{gzip_flag, skip_invalid_flag};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ExplodeConfig {
    /// Skip invalid records that can't be decoded.
    pub(crate) skip_invalid: Option<bool>,

    /// Compress output in gzip format
    pub(crate) gzip: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Explode {
    /// Skip invalid records that can't be decoded.
    #[arg(short, long)]
    skip_invalid: bool,

    /// Compress each partition in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// A filter expression used for searching
    #[arg(long = "where")]
    filter: Option<String>,

    /// Connects the where clause with additional expressions using the
    /// logical AND-operator (conjunction)
    ///
    /// This option can't be combined with `--or` or `--not`.
    #[arg(long, requires = "filter", conflicts_with_all = ["or", "not"])]
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
    #[arg(long, requires = "filter", conflicts_with_all = ["and", "or"])]
    not: Vec<String>,

    /// When this flag is provided, comparison operations will be
    /// search case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons
    /// (range: 0.0..1.0)
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
        default_value = "75")]
    strsim_threshold: u8,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to <OUTPUT> instead of stdout
    #[arg(short, long)]
    output: Option<OsString>,

    /// Split a record by level (main, local, copy).
    level: Level,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

macro_rules! record_bytes {
    ($fields:expr) => {{
        let mut buffer = Vec::<u8>::new();
        for field in $fields.iter() {
            let _ = field.write_to(&mut buffer);
        }
        buffer.push(b'\n');
        buffer
    }};
}

macro_rules! push_record {
    ($records:expr, $main:expr, $local:expr, $acc:expr) => {
        if !$acc.is_empty() {
            let mut record = $main.clone();
            if let Some(local) = $local {
                record.push(local);
            }
            record.extend_from_slice(&$acc);

            $records.push(record);
            $acc.clear();
        }
    };

    ($records:expr, $main:expr, $acc:expr) => {
        if !$acc.is_empty() {
            let mut record = $main.clone();
            record.extend_from_slice(&$acc);
            $records.push(record);
            $acc.clear();
        }
    };
}

fn process_main(
    record: &ByteRecord,
    matcher: Option<&RecordMatcher>,
    options: &MatcherOptions,
    writer: &mut Box<dyn ByteRecordWrite>,
) -> io::Result<()> {
    if let Some(matcher) = matcher {
        if !matcher.is_match(record, options) {
            return Ok(());
        }
    }

    writer.write_byte_record(record)
}

fn process_copy(
    record: &ByteRecord,
    matcher: Option<&RecordMatcher>,
    options: &MatcherOptions,
    writer: &mut Box<dyn ByteRecordWrite>,
) -> io::Result<()> {
    let mut main = vec![];
    let mut acc = vec![];
    let mut records = vec![];
    let mut local = None;
    let mut count = None;

    for field in record.iter() {
        match field.level() {
            Level::Main => main.push(field),
            Level::Local => {
                push_record!(records, main, local, acc);
                local = Some(field);
                count = None;
            }
            Level::Copy => {
                if count != field.occurrence() {
                    push_record!(records, main, local, acc);
                    count = field.occurrence();
                }

                acc.push(field);
            }
        }
    }

    push_record!(records, main, local, acc);
    for fields in records {
        let data = record_bytes!(fields);
        let record =
            ByteRecord::from_bytes(&data).expect("valid record");

        if let Some(matcher) = matcher {
            if !matcher.is_match(&record, options) {
                continue;
            }
        }

        writer.write_byte_record(&record)?;
    }

    Ok(())
}

fn process_local(
    record: &ByteRecord,
    matcher: Option<&RecordMatcher>,
    options: &MatcherOptions,
    writer: &mut Box<dyn ByteRecordWrite>,
) -> io::Result<()> {
    let mut main = vec![];
    let mut acc = vec![];
    let mut records = vec![];

    for field in record.iter() {
        match field.level() {
            Level::Main => main.push(field),
            Level::Copy => acc.push(field),
            Level::Local => {
                push_record!(records, main, acc);
                acc.push(field)
            }
        }
    }

    push_record!(records, main, acc);

    for fields in records.iter() {
        let data = record_bytes!(fields);
        let record = ByteRecord::from_bytes(&data).unwrap();

        if let Some(matcher) = matcher {
            if !matcher.is_match(&record, options) {
                continue;
            }
        }

        writer.write_byte_record(&record)?;
    }

    Ok(())
}

impl Explode {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let gzip_compression = gzip_flag!(self.gzip, config.explode);
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.explode,
            config.global
        );

        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100.0)
            .case_ignore(self.ignore_case);

        let nf = if let Some(ref global) = config.global {
            global.translit
        } else {
            None
        };

        let filter = self
            .filter
            .map(|value| NormalizationForm::translit_opt(value, nf));

        let not: Vec<_> = self
            .not
            .iter()
            .map(|value| NormalizationForm::translit_opt(value, nf))
            .collect();

        let and: Vec<_> = self
            .and
            .iter()
            .map(|value| NormalizationForm::translit_opt(value, nf))
            .collect();

        let or: Vec<_> = self
            .or
            .iter()
            .map(|value| NormalizationForm::translit_opt(value, nf))
            .collect();

        let matcher = if let Some(ref matcher_str) = filter {
            let mut matcher = RecordMatcher::try_from(matcher_str)?;

            for matcher_str in and.iter() {
                matcher =
                    matcher & RecordMatcher::try_from(matcher_str)?;
            }

            for matcher_str in or.iter() {
                matcher =
                    matcher | RecordMatcher::try_from(matcher_str)?;
            }

            for matcher_str in not.iter() {
                matcher =
                    matcher & !RecordMatcher::try_from(matcher_str)?;
            }

            Some(matcher)
        } else {
            None
        };

        let mut progress = Progress::new(self.progress);

        let mut writer = WriterBuilder::new()
            .gzip(gzip_compression)
            .from_path_or_stdout(self.output)?;

        let process_record = match self.level {
            Level::Main => process_main,
            Level::Copy => process_copy,
            Level::Local => process_local,
        };

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next() {
                if let Err(e) = result {
                    if e.is_invalid_record() && skip_invalid {
                        progress.invalid();
                        continue;
                    } else {
                        return Err(e.into());
                    }
                }

                let record = result.unwrap();
                progress.record();
                process_record(
                    &record,
                    matcher.as_ref(),
                    &options,
                    &mut writer,
                )?;
            }
        }

        progress.record();
        writer.finish()?;
        Ok(())
    }
}
