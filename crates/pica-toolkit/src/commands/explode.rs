use std::ffi::OsString;
use std::io;

use clap::{value_parser, Parser};
use pica_matcher::{MatcherBuilder, MatcherOptions, RecordMatcher};
use pica_record::io::{
    ByteRecordWrite, ReaderBuilder, RecordsIterator, WriterBuilder,
};
use pica_record::{ByteRecord, Level};
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

    /// Limit the result to first <n> records
    ///
    /// Note: A limit value `0` means no limit.
    #[arg(long, short, value_name = "n", default_value = "0")]
    limit: usize,

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
            if !$local.is_empty() {
                record.extend_from_slice(&$local);
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
    limit: &mut usize,
) -> io::Result<bool> {
    if *limit == 0 {
        return Ok(false);
    }

    if let Some(matcher) = matcher {
        if !matcher.is_match(record, options) {
            return Ok(true);
        }
    }

    writer.write_byte_record(record)?;
    *limit -= 1;

    Ok(true)
}

fn process_copy(
    record: &ByteRecord,
    matcher: Option<&RecordMatcher>,
    options: &MatcherOptions,
    writer: &mut Box<dyn ByteRecordWrite>,
    limit: &mut usize,
) -> io::Result<bool> {
    debug_assert!(*limit > 0);

    let mut last = Level::Main;
    let mut records = vec![];
    let mut main = vec![];
    let mut local = vec![];
    let mut copy = vec![];
    let mut count = None;

    for field in record.iter() {
        match field.level() {
            Level::Main => main.push(field),
            Level::Local => {
                if last == Level::Copy {
                    push_record!(records, main, local, copy);
                    local.clear();
                    count = None;
                }

                local.push(field);
            }
            Level::Copy => {
                if count != field.occurrence() {
                    push_record!(records, main, local, copy);
                    count = field.occurrence();
                }

                copy.push(field);
            }
        }

        last = field.level();
    }

    push_record!(records, main, local, copy);

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
        *limit -= 1;

        if *limit == 0 {
            return Ok(false);
        }
    }

    Ok(*limit == 0)
}

fn process_local(
    record: &ByteRecord,
    matcher: Option<&RecordMatcher>,
    options: &MatcherOptions,
    writer: &mut Box<dyn ByteRecordWrite>,
    limit: &mut usize,
) -> io::Result<bool> {
    debug_assert!(*limit > 0);

    let mut main = vec![];
    let mut acc = vec![];
    let mut records = vec![];
    let mut last = Level::Main;

    for field in record.iter() {
        match field.level() {
            Level::Main => main.push(field),
            Level::Copy => acc.push(field),
            Level::Local => {
                if last == Level::Copy {
                    push_record!(records, main, acc);
                }

                acc.push(field)
            }
        }

        last = field.level();
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
        *limit -= 1;

        if *limit == 0 {
            return Ok(false);
        }
    }

    Ok(*limit == 0)
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

        let matcher = if let Some(matcher) = self.filter {
            Some(
                MatcherBuilder::new(matcher, nf)?
                    .and(self.and)?
                    .not(self.not)?
                    .or(self.or)?
                    .build(),
            )
        } else {
            None
        };

        let mut progress = Progress::new(self.progress);

        let mut limit = if self.limit == 0 {
            usize::MAX
        } else {
            self.limit
        };

        let mut writer = WriterBuilder::new()
            .gzip(gzip_compression)
            .from_path_or_stdout(self.output)?;

        let process_record = match self.level {
            Level::Main => process_main,
            Level::Copy => process_copy,
            Level::Local => process_local,
        };

        'outer: for filename in self.filenames {
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
                let stop = process_record(
                    &record,
                    matcher.as_ref(),
                    &options,
                    &mut writer,
                    &mut limit,
                )?;

                if stop {
                    break 'outer;
                }
            }
        }

        progress.record();
        writer.finish()?;
        Ok(())
    }
}
