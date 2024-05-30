use std::ffi::OsString;

use clap::{value_parser, Parser};
use pica_matcher::{MatcherBuilder, MatcherOptions};
use pica_record::io::{ReaderBuilder, RecordsIterator, WriterBuilder};
use pica_record::{ByteRecord, FieldRef, Level};
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

    /// Keep only fields specified by a list of predicates.
    #[arg(long, short)]
    keep: Option<String>,

    /// Discard fields specified by a list of predicates.
    #[arg(long, short)]
    discard: Option<String>,

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

macro_rules! push_record {
    ($records:expr, $main:expr, $local:expr, $acc:expr) => {
        if !$acc.is_empty() {
            let mut record = $main.clone();
            if !$local.is_empty() {
                record.extend_from_slice(&$local);
            }
            record.extend_from_slice(&$acc);
            $records.push(record);
        }
    };

    ($records:expr, $main:expr, $acc:expr) => {
        if !$acc.is_empty() {
            let mut record = $main.clone();
            record.extend_from_slice(&$acc);
            $records.push(record);
        }
    };
}

#[inline]
fn process_main<'a>(
    record: &'a ByteRecord<'a>,
) -> Vec<Vec<&'a FieldRef<'a>>> {
    vec![record.iter().collect()]
}

fn process_local<'a>(
    record: &'a ByteRecord<'a>,
) -> Vec<Vec<&FieldRef<'a>>> {
    let mut iter = record.iter().peekable();
    let mut records = vec![];
    let mut main = vec![];
    let mut acc = vec![];

    while let Some(cur) = iter.next() {
        match cur.level() {
            Level::Main => main.push(cur),
            Level::Local => acc.push(cur),
            Level::Copy => {
                acc.push(cur);

                if let Some(next) = iter.peek() {
                    if next.level() == Level::Local {
                        push_record!(records, main, acc);
                        acc.clear();
                    }
                }
            }
        }
    }

    push_record!(records, main, acc);
    records
}

fn process_copy<'a>(
    record: &'a ByteRecord<'a>,
) -> Vec<Vec<&FieldRef<'a>>> {
    let mut iter = record.iter().peekable();
    let mut records = vec![];
    let mut main = vec![];
    let mut local = vec![];
    let mut copy = vec![];
    let mut count = None;

    while let Some(cur) = iter.next() {
        match cur.level() {
            Level::Main => main.push(cur),
            Level::Local => {
                local.push(cur);
            }
            Level::Copy => {
                if count != cur.occurrence() {
                    push_record!(records, main, local, copy);
                    count = cur.occurrence();
                    copy.clear();
                }

                copy.push(cur);

                if let Some(next) = iter.peek() {
                    if next.level() == Level::Local {
                        push_record!(records, main, local, copy);
                        count = None;
                        local.clear();
                        copy.clear();
                    }
                }
            }
        }
    }

    push_record!(records, main, local, copy);
    records
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
        let mut count = 0;

        let mut writer = WriterBuilder::new()
            .gzip(gzip_compression)
            .from_path_or_stdout(self.output)?;

        let process = match self.level {
            Level::Main => process_main,
            Level::Local => process_local,
            Level::Copy => process_copy,
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

                for record in process(&record) {
                    let mut data = Vec::<u8>::new();
                    for field in record.iter() {
                        let _ = field.write_to(&mut data);
                    }
                    data.push(b'\n');

                    let record = ByteRecord::from_bytes(&data)
                        .expect("valid record");

                    if let Some(ref matcher) = matcher {
                        if !matcher.is_match(&record, &options) {
                            continue;
                        }
                    }

                    writer.write_byte_record(&record)?;
                    count += 1;

                    if self.limit > 0 && count >= self.limit {
                        break 'outer;
                    }
                }
            }
        }

        progress.record();
        writer.finish()?;
        Ok(())
    }
}
