use std::ffi::OsString;
use std::fs::read_to_string;
use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;

use clap::{value_parser, Parser};
use lazy_static::lazy_static;
use pica::matcher::{
    MatcherFlags, OccurrenceMatcher, RecordMatcher, TagMatcher,
};
use pica::{Path, PicaWriter, Reader, ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::common::FilterList;
use crate::translit::translit_maybe2;
use crate::util::{CliError, CliResult};
use crate::{gzip_flag, skip_invalid_flag, Config};

lazy_static! {
    static ref IDN_PATH: Path = Path::from_str("003@.0").unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct FilterConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Filter {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Filter only records that did not match
    #[arg(long, short = 'v')]
    invert_match: bool,

    /// When this flag is provided, comparision operations will be
    /// search case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons
    /// (range: 0.0..1.0)
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
        default_value = "75")]
    strsim_threshold: u8,

    /// Reduce the record to fields which are specified in <REDUCE>
    #[arg(long, short = 'R', default_value = "")]
    reduce: String,

    /// Take filter expressions from <EXPR_FILE>
    #[arg(long = "file", short = 'f')]
    expr_file: Option<PathBuf>,

    /// Ignore records which are *not* explicitly listed in one of the
    /// given allow-lists. An allow-list must be an CSV, whereby the
    /// first column contains the IDN (003@.0).
    #[arg(long, short = 'A')]
    allow_list: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists. An deny-list must be an CSV, whereby the
    /// first column contains the IDN (003@.0).
    #[arg(long, short = 'D')]
    deny_list: Vec<PathBuf>,

    /// Limit the result to first <n> records
    #[arg(long, short, value_name = "n", default_value = "0")]
    limit: usize,

    /// Connects the filter with additional expressions using the
    /// logical AND-operator (conjunction)
    #[arg(long, conflicts_with_all = ["or", "not"])]
    and: Vec<String>,

    /// Connects the filter with additional expressions using the
    /// logical OR-operator (disjunction)
    #[arg(long, conflicts_with_all = ["and", "not"])]
    or: Vec<String>,

    /// Connects the filter with additional expressions using the
    /// logical NOT-operator (negation)
    #[arg(long, conflicts_with_all = ["and", "or"])]
    not: Vec<String>,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// Write simultaneously to the file <filename> and stdout
    #[arg(long, value_name = "filename", conflicts_with = "output")]
    tee: Option<PathBuf>,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// A filter expression used for searching
    filter: String,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Filter {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        // let limit = self.limit.unwrap_or_default();
        let gzip_compression = gzip_flag!(self.gzip, config.filter);
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.filter,
            config.global
        );

        let mut writer: Box<dyn PicaWriter> = WriterBuilder::new()
            .gzip(gzip_compression)
            .append(self.append)
            .from_path_or_stdout(self.output)?;

        let mut tee_writer = match self.tee {
            Some(path) => Some(
                WriterBuilder::new()
                    .gzip(gzip_compression)
                    .append(self.append)
                    .from_path(path)?,
            ),
            None => None,
        };

        let items = self
            .reduce
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let mut reducers = vec![];
        for item in items {
            let (tag, occ) = if let Some(pos) = item.rfind('/') {
                (&item[0..pos], &item[pos..])
            } else {
                (item, "/*")
            };

            let tag = TagMatcher::new(tag).map_err(|_| {
                CliError::Other("invalid reduce value".to_string())
            })?;

            let occ = OccurrenceMatcher::new(occ).map_err(|_| {
                CliError::Other("invalid reduce value".to_string())
            })?;

            reducers.push((tag, occ));
        }

        let filter_str = if let Some(filename) = self.expr_file {
            read_to_string(filename).unwrap()
        } else {
            self.filter
        };

        let filter_str = if let Some(ref global) = config.global {
            translit_maybe2(&filter_str, global.translit)
        } else {
            filter_str
        };

        let mut filter = match RecordMatcher::new(&filter_str) {
            Ok(f) => f,
            _ => {
                return Err(CliError::Other(format!(
                    "invalid filter: \"{filter_str}\""
                )))
            }
        };

        if !self.and.is_empty() {
            let predicates = self
                .and
                .iter()
                .map(RecordMatcher::new)
                .collect::<Result<Vec<_>, _>>()?;

            for expr in predicates.into_iter() {
                filter = filter & expr;
            }
        }

        if !self.not.is_empty() {
            let predicates = self
                .not
                .iter()
                .map(RecordMatcher::new)
                .collect::<Result<Vec<_>, _>>()?;

            for expr in predicates.into_iter() {
                filter = filter & !expr;
            }
        }

        if !self.or.is_empty() {
            let predicates = self
                .or
                .iter()
                .map(RecordMatcher::new)
                .collect::<Result<Vec<_>, _>>()?;

            for expr in predicates.into_iter() {
                filter = filter | expr;
            }
        }

        let allow_list = if !self.allow_list.is_empty() {
            FilterList::new(self.allow_list)?
        } else {
            FilterList::default()
        };

        let deny_list = if !self.deny_list.is_empty() {
            FilterList::new(self.deny_list)?
        } else {
            FilterList::default()
        };

        let mut count = 0;
        let flags = MatcherFlags {
            strsim_threshold: self.strsim_threshold as f64 / 100.0,
            ignore_case: self.ignore_case,
        };

        for filename in self.filenames {
            let builder =
                ReaderBuilder::new().skip_invalid(skip_invalid);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
            {
                Some("-") => builder.from_reader(Box::new(io::stdin())),
                _ => builder.from_path(filename)?,
            };

            for result in reader.byte_records() {
                let mut record = result?;
                let idn = record.path(&IDN_PATH);
                let idn = idn.first();

                if !allow_list.is_empty() {
                    if let Some(idn) = idn {
                        if !allow_list.contains(*idn) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                if !deny_list.is_empty() {
                    if let Some(idn) = idn {
                        if deny_list.contains(*idn) {
                            continue;
                        }
                    }
                }

                let mut is_match = filter.is_match(&record, &flags);
                if self.invert_match {
                    is_match = !is_match;
                }

                if is_match {
                    if !reducers.is_empty() {
                        record.reduce(&reducers);
                    }

                    writer.write_byte_record(&record)?;

                    if let Some(ref mut writer) = tee_writer {
                        writer.write_byte_record(&record)?;
                    }

                    count += 1;
                }

                if self.limit > 0 && count >= self.limit {
                    break;
                }
            }
        }

        writer.finish()?;
        if let Some(ref mut writer) = tee_writer {
            writer.finish()?;
        }

        Ok(())
    }
}
