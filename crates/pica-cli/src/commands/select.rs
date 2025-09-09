use std::collections::hash_map::DefaultHasher;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;
use hashbrown::HashSet;
use pica_record::prelude::*;

use crate::prelude::*;
use crate::utils::FilterSet;

/// Select subfield values from records
#[derive(Parser, Debug)]
pub(crate) struct Select {
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

    /// Write output tab-separated (TSV)
    #[arg(long, short)]
    tsv: bool,

    /// Transliterate output into the selected normal form NF
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc")
    #[arg(long = "translit", value_name = "NF")]
    nf: Option<NormalizationForm>,

    /// Comma-separated list of column names
    #[arg(long, short = 'H')]
    header: Option<String>,

    /// Append to the given file, do not overwrite
    #[arg(long)]
    append: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// Query (comma-separated list of path expressions or string
    /// literals)
    query: String,

    /// Read one or more files in normalized PICA+ format. If no
    /// filenames where given or a filename is "-", data is read from
    /// standard input (stdin)
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    #[command(flatten, next_help_heading = "Filter options")]
    pub(crate) filter_opts: FilterOpts,
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
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut seen = HashSet::new();
        let mut count = 0;

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let options = QueryOptions::default()
            .strsim_threshold(
                self.filter_opts.strsim_threshold as f64 / 100f64,
            )
            .case_ignore(self.filter_opts.ignore_case)
            .separator(self.separator)
            .squash(self.squash)
            .merge(self.merge);

        let matcher_options = MatcherOptions::from(&options);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;

        let translit = translit(config.normalization.clone());
        let query = Query::new(translit(self.query))?;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(if self.tsv { b'\t' } else { b',' })
            .from_writer(writer(self.output, self.append)?);

        if let Some(header) = self.header {
            writer.write_record(header.split(',').map(str::trim))?;
        }

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

                        if !filter_set.check(record) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher
                            && !matcher
                                .is_match(record, &matcher_options)
                        {
                            continue;
                        }

                        let outcome = record.query(&query, &options);
                        for row in outcome.iter() {
                            if self.no_empty_columns
                                && row.iter().any(|e| e.is_empty())
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

                            if !row.iter().all(|e| e.is_empty()) {
                                if self.nf.is_none() {
                                    writer.write_record(row)?;
                                } else {
                                    writer.write_record(
                                        row.iter().map(|s| {
                                            (crate::translit::translit(
                                                self.nf.clone(),
                                            ))(
                                                s.to_string()
                                            )
                                        }),
                                    )?;
                                };
                            }
                        }

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
