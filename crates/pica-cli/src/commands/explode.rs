use std::ffi::OsString;
use std::process::ExitCode;

use clap::Parser;
use pica_record::prelude::*;
use pica_record::primitives::{FieldRef, Level};

use crate::prelude::*;

/// Split records at main, local or copy level.
#[derive(Parser, Debug)]
pub(crate) struct Explode {
    /// Compress each partition in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Keep only fields specified by a list of predicates.
    #[arg(long, short)]
    keep: Option<String>,

    /// Discard fields specified by a list of predicates.
    #[arg(long, short)]
    discard: Option<String>,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// Split a record by level (main, local, copy).
    level: Level,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    #[command(flatten, next_help_heading = "Filter options")]
    pub(crate) filter_opts: FilterOpts,
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

#[inline(always)]
fn process_main<'a>(
    record: &'a ByteRecord<'a>,
) -> Vec<Vec<&'a FieldRef<'a>>> {
    vec![record.fields().iter().collect()]
}

fn process_local<'a>(
    record: &'a ByteRecord<'a>,
) -> Vec<Vec<&'a FieldRef<'a>>> {
    let mut iter = record.fields().iter().peekable();
    let mut records = vec![];
    let mut main = vec![];
    let mut acc = vec![];

    while let Some(cur) = iter.next() {
        match cur.level() {
            Level::Main => main.push(cur),
            Level::Local => acc.push(cur),
            Level::Copy => {
                acc.push(cur);

                if let Some(next) = iter.peek()
                    && next.level() == Level::Local
                {
                    push_record!(records, main, acc);
                    acc.clear();
                }
            }
        }
    }

    push_record!(records, main, acc);
    records
}

fn process_copy<'a>(
    record: &'a ByteRecord<'a>,
) -> Vec<Vec<&'a FieldRef<'a>>> {
    let mut iter = record.fields().iter().peekable();
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

                if let Some(next) = iter.peek()
                    && next.level() == Level::Local
                {
                    push_record!(records, main, local, copy);
                    count = None;
                    local.clear();
                    copy.clear();
                }
            }
        }
    }

    push_record!(records, main, local, copy);
    records
}

impl Explode {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let discard = parse_predicates(self.discard)?;
        let keep = parse_predicates(self.keep)?;

        let mut data = Vec::<u8>::new();
        let mut count = 0;

        let mut writer = WriterBuilder::new()
            .gzip(self.gzip)
            .from_path_or_stdout(self.output)?;

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let options = MatcherOptions::from(&self.filter_opts);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;

        let process = match self.level {
            Level::Main => process_main,
            Level::Local => process_local,
            Level::Copy => process_copy,
        };

        'outer: for path in self.filenames {
            let mut reader = ReaderBuilder::new().from_path(path)?;

            while let Some(result) = reader.next_byte_record() {
                match result {
                    Err(e) if e.skip_parse_err(skip_invalid) => {
                        progress.update(true);
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                    Ok(ref record) => {
                        progress.update(false);

                        for record in process(record) {
                            data.clear();

                            for field in record.iter() {
                                let _ = field.write_to(&mut data);
                            }
                            data.push(b'\n');

                            let mut record =
                                ByteRecord::from_bytes(&data).unwrap();

                            if !filter_set.check(&record) {
                                continue;
                            }

                            if let Some(ref matcher) = matcher
                                && !matcher.is_match(&record, &options)
                            {
                                continue;
                            }

                            if !keep.is_empty() {
                                record.retain(|field| {
                                    for (t, o) in keep.iter() {
                                        if t.is_match(field.tag())
                                            && o.is_match(
                                                field.occurrence(),
                                            )
                                        {
                                            return true;
                                        }
                                    }

                                    false
                                });
                            }

                            if !discard.is_empty() {
                                record.retain(|field| {
                                    for (t, o) in discard.iter() {
                                        if t.is_match(field.tag())
                                            && o.is_match(
                                                field.occurrence(),
                                            )
                                        {
                                            return false;
                                        }
                                    }

                                    true
                                });
                            }

                            writer.write_byte_record(&record)?;
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
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
