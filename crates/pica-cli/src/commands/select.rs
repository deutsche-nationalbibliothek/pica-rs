use std::collections::hash_map::DefaultHasher;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{value_parser, Parser};
use hashbrown::HashSet;
use pica_record::prelude::*;

use crate::prelude::*;

/// Select subfield values from records
#[derive(Parser, Debug)]
pub(crate) struct Select {
    /// Whether to skip invalid records or not.
    #[arg(short, long)]
    skip_invalid: bool,

    /// Whether to squash all values of a repeated subfield into a
    /// single value or not. The separator can be specified by the
    /// `--separator` option.
    ///
    /// Note: This option cannot be used with `--merge`.
    #[arg(long, conflicts_with = "merge")]
    squash: bool,

    /// Whether to merge all values of a column into a single value or
    /// not. The separator can be specified by the `--separator`
    ///
    /// Note: This option cannot be used with `--squash`.
    #[arg(long, conflicts_with = "squash")]
    merge: bool,

    /// Sets the separator used for squashing of repeated subfield
    /// values into a single value. Note that it's possible to use the
    /// empty string as a separator.
    #[arg(long, default_value = "|")]
    separator: String,

    /// Disallow empty columns
    #[arg(long)]
    no_empty_columns: bool,

    /// Skip duplicate rows
    #[arg(long, short)]
    unique: bool,

    /// When this flag is provided, comparison operations will be
    /// search case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons (0 <= score
    /// < 100).
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
          default_value = "75")]
    strsim_threshold: u8,

    /// Write output tab-separated (TSV)
    #[arg(long, short)]
    tsv: bool,

    /// Transliterate output into the selected normal form <NF>
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc")
    #[arg(long = "translit", value_name = "NF")]
    nf: Option<NormalizationForm>,

    /// Comma-separated list of column names
    #[arg(long, short = 'H')]
    header: Option<String>,

    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// Limit the result to first <n> records (not rows!)
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

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Query (comma-separated list of path expressions or string
    /// literals)
    query: String,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

fn writer(
    filename: Option<OsString>,
    append: bool,
) -> io::Result<Box<dyn Write>> {
    Ok(match filename {
        Some(filename) => Box::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(!append)
                .append(append)
                .open(filename)?,
        ),
        None => Box::new(io::stdout().lock()),
    })
}

impl Select {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut seen = HashSet::new();
        let mut count = 0;

        let options = QueryOptions::default()
            .strsim_threshold(self.strsim_threshold as f64 / 100f64)
            .case_ignore(self.ignore_case)
            .separator(self.separator)
            .squash(self.squash)
            .merge(self.merge);

        let matcher_options = MatcherOptions::from(&options);
        let matcher = if let Some(matcher) = self.filter {
            Some(
                RecordMatcherBuilder::with_transform(
                    matcher,
                    translit(config.normalization.as_ref()),
                )?
                .and(self.and)?
                .not(self.not)?
                .or(self.or)?
                .build(),
            )
        } else {
            None
        };

        let translit = translit(config.normalization.as_ref());
        let query = Query::new(translit(self.query))?;
        let filter_set = FilterSet::new(self.allow, self.deny)?;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer(self.output, self.append)?);

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(str::trim))?;
        }

        'outer: for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next_string_record() {
                match result {
                    Err(e) if e.skip_parse_err(skip_invalid) => {
                        progress.update(true);
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                    Ok(ref record) => {
                        progress.update(false);

                        if !filter_set.check(record.ppn().into()) {
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
                        for row in outcome.iter() {
                            if self.no_empty_columns
                                && row.iter().any(String::is_empty)
                            {
                                continue;
                            }

                            if self.unique {
                                let mut hasher = DefaultHasher::new();
                                row.hash(&mut hasher);
                                let hash = hasher.finish();

                                if seen.contains(&hash) {
                                    continue;
                                }

                                seen.insert(hash);
                            }

                            if !row.iter().all(String::is_empty) {
                                if self.nf.is_none() {
                                    writer.write_record(row)?;
                                } else {
                                    writer.write_record(
                                        row.iter().map(|s| {
                                            (crate::translit::translit(
                                                self.nf.as_ref(),
                                            ))(
                                                s
                                            )
                                        }),
                                    )?;
                                };
                            }
                        }

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
