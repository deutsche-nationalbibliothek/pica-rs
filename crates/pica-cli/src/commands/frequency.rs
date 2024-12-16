use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use bstr::BString;
use clap::{value_parser, Parser};
use hashbrown::{HashMap, HashSet};
use pica_record::prelude::*;

use crate::prelude::*;

/// Compute a frequency table of a subfield
///
/// This command computes a frequency table over all subfield values of
/// the given path expression. By default, the resulting frequency table
/// is sorted in descending order by default (the most frequent value is
/// printed first). If the count of two or more subfield values is
/// equal, these lines are given in lexicographical order.
///
/// The set of fields, which are included in the result of a record, can
/// be restricted by an optional subfield filter. A subfield filter
/// requires the {}-notation and is expected at the first position (e.g.
/// "044H/*{b == 'GND' && 9?, 9}").
#[derive(Parser, Debug)]
pub(crate) struct Frequency {
    /// Skip invalid records that can't be decoded as normalized PICA+.
    #[arg(long, short)]
    skip_invalid: bool,

    /// When this flag is set, comparison operations will be search
    /// case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons (0 <= score
    /// < 100).
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
          default_value = "75")]
    strsim_threshold: u8,

    /// Skip duplicate rows (of a record).
    #[arg(long, short)]
    unique: bool,

    /// Sort results in reverse order.
    #[arg(long, short)]
    reverse: bool,

    /// Ignore records which are *not* explicitly listed in one of the
    /// given allow-lists.
    ///
    /// A allow-list must be an CSV/TSV or Apache Arrow file, whereby
    /// a column `idn` exists. If the file extension is `.feather`,
    /// `.arrow`, or `.ipc` the file is automatically interpreted
    /// as Apache Arrow; file existions `.csv`, `.csv.gz`, `.tsv` or
    /// `.tsv.gz` is interpreted as CSV/TSV.
    #[arg(long = "allow-list", short = 'A')]
    allow: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists.
    ///
    /// A deny-list must be an CSV/TSV or Apache Arrow file, whereby
    /// a column `idn` exists. If the file extension is `.feather`,
    /// `.arrow`, or `.ipc` the file is automatically interpreted
    /// as Apache Arrow; file existions `.csv`, `.csv.gz`, `.tsv` or
    /// `.tsv.gz` is interpreted as CSV/TSV.
    #[arg(long = "deny-list", short = 'D')]
    deny: Vec<PathBuf>,

    /// Limit result to the N most frequent subfield values.
    #[arg(
        long,
        short,
        value_name = "N",
        hide_default_value = true,
        default_value = "0"
    )]
    limit: usize,

    /// Ignore rows with a frequency < VALUE.
    #[arg(
        long,
        value_name = "VALUE",
        default_value = "0",
        hide_default_value = true
    )]
    threshold: u64,

    /// A filter expression used for searching
    #[arg(long = "where")]
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
    /// This option can't be combined with `--or`.
    #[arg(long, requires = "filter", conflicts_with = "or")]
    not: Vec<String>,

    /// Comma-separated list of column names.
    #[arg(long, short = 'H')]
    header: Option<String>,

    /// Write output tab-separated (TSV)
    #[arg(long, short)]
    tsv: bool,

    /// Transliterate output into the selected normal form NF
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc").
    #[arg(long = "translit", value_name = "NF")]
    nf: Option<NormalizationForm>,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to FILENAME instead of stdout.
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// Query (comma-separated list of path expressions or string
    /// literals)
    query: String,

    /// Read one or more files in normalized PICA+ format. With no
    /// files, or when a filename is '-', read from standard input
    /// (stdin).
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Frequency {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut seen = HashSet::new();

        let translit =
            crate::translit::translit(config.normalization.as_ref());
        let query = Query::new(translit(self.query))?;
        let matcher = if let Some(matcher) = self.filter {
            Some(
                RecordMatcherBuilder::with_transform(
                    matcher, translit,
                )?
                .and(self.and)?
                .not(self.not)?
                .or(self.or)?
                .build(),
            )
        } else {
            None
        };

        let mut ftable: HashMap<Vec<BString>, u64> = HashMap::new();
        let options = QueryOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100f64)
            .case_ignore(self.ignore_case);

        let filter_set = FilterSet::new(self.allow, self.deny)?;
        let matcher_options = MatcherOptions::from(&options);

        let writer: Box<dyn Write> = match self.output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout()),
        };

        let mut writer = csv::WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer);

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
                    Ok(ref record) => {
                        progress.update(false);

                        if !filter_set.check(record.ppn()) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher {
                            if !matcher
                                .is_match(record, &matcher_options)
                            {
                                continue;
                            }
                        }

                        let outcome = record.query(&query, &options);
                        seen.clear();

                        for key in outcome.clone().into_iter() {
                            if key.iter().any(|e| !e.is_empty()) {
                                if self.unique {
                                    if seen.contains(&key) {
                                        continue;
                                    }

                                    seen.insert(key.clone());
                                }

                                *ftable.entry(key).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(str::trim))?;
        }

        let mut ftable_sorted: Vec<(&Vec<BString>, &u64)> =
            ftable.iter().collect();

        if self.reverse {
            ftable_sorted.sort_by(|a, b| match a.1.cmp(b.1) {
                Ordering::Equal => a.0.cmp(b.0),
                ordering => ordering,
            });
        } else {
            ftable_sorted.sort_by(|a, b| match b.1.cmp(a.1) {
                Ordering::Equal => a.0.cmp(b.0),
                ordering => ordering,
            });
        }

        let translit = crate::translit::translit(self.nf.as_ref());
        for (i, (values, freq)) in ftable_sorted.iter().enumerate() {
            if self.limit > 0 && i >= self.limit {
                break;
            }

            if **freq < self.threshold {
                break;
            }

            let mut record = values
                .iter()
                .map(ToString::to_string)
                .map(&translit)
                .collect::<Vec<_>>();

            record.push(freq.to_string());
            writer.write_record(record)?;
        }

        progress.finish();
        writer.flush()?;

        Ok(ExitCode::SUCCESS)
    }
}
