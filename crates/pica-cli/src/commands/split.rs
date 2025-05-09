use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, value_parser};
use pica_record::prelude::*;

use crate::prelude::*;

/// Splits a list of records into chunks
///
/// This command is used to split a list of records into chunks of a
/// given size. To write all chunks in a directory, use the `--outdir`
/// or `-o` option (if the directory doesn't exist, the directory will
/// be created automatically).
#[derive(Parser, Debug)]
pub(crate) struct Split {
    /// Skip invalid records that can't be decoded as normalized PICA+
    #[arg(short, long)]
    skip_invalid: bool,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long)]
    progress: bool,

    /// When this flag is provided, comparison operations will be
    /// search case insensitive
    #[arg(long, short)]
    ignore_case: bool,

    /// The minimum score for string similarity comparisons (0 <= score
    /// < 100).
    #[arg(long, value_parser = value_parser!(u8).range(0..100),
          default_value = "75")]
    strsim_threshold: u8,

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
    /// An allow-list must be an CSV, TSV or Apache Arrow file. By
    /// default the column `ppn` or `idn` is used to get the values
    /// of the allow list. These values are compared against the PPN
    /// (003@.0) of record.
    ///
    /// The column name can be changed using the `--filter-set-column`
    /// option and the path to the comparison values can be changed
    /// with option `--filter-set-source`.
    ///
    /// # Note
    ///
    /// If the allow list is empty, all records are blocked. With more
    /// than one allow list, the filter set is made up of all partial
    /// lists. lists.
    #[arg(long = "allow-list", short = 'A')]
    allow: Vec<PathBuf>,

    /// Ignore records which are explicitly listed in one of the
    /// given deny-lists.
    ///
    /// An deny-list must be an CSV, TSV or Apache Arrow file. By
    /// default the column `ppn` or `idn` is used to get the values
    /// of the allow list. These values are compared against the PPN
    /// (003@.0) of record.
    ///
    /// The column name can be changed using the `--filter-set-column`
    /// option and the path to the comparison values can be changed
    /// with option `--filter-set-source`.
    ///
    /// # Note
    ///
    /// With more than one allow list, the filter set is made up of all
    /// partial lists.
    #[arg(long = "deny-list", short = 'D')]
    deny: Vec<PathBuf>,

    /// Defines the column name of an allow or deny list. By default,
    /// the column `ppn` is used or, if this is not available, the
    /// column `idn` is used.
    #[arg(long, value_name = "COLUMN")]
    filter_set_column: Option<String>,

    /// Defines an optional path to the comparison values of the
    /// record. If no path is specified, a comparison with the PPN in
    /// field 003@.0 is assumed.
    #[arg(long, value_name = "PATH")]
    filter_set_source: Option<Path>,

    /// Write partitions into OUTDIR
    #[arg(long, short, value_name = "outdir", default_value = ".")]
    outdir: PathBuf,

    /// Filename template ("{}" is replaced by the chunk number)
    #[arg(long, value_name = "template")]
    template: Option<String>,

    /// Chunk size
    #[arg(value_parser = value_parser!(u32).range(1..))]
    chunk_size: u32,

    /// Read one or more files in normalized PICA+ format
    ///
    /// If no filenames where given or a filename is "-", data is read
    /// from standard input (stdin).
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Split {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut chunks: u32 = 0;
        let mut count = 0;

        let filter_set = FilterSetBuilder::new()
            .source(self.filter_set_source)
            .column(self.filter_set_column)
            .allow(self.allow)
            .deny(self.deny)
            .build()?;

        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100.0)
            .case_ignore(self.ignore_case);

        let matcher = if let Some(matcher) = self.filter {
            Some(
                RecordMatcherBuilder::with_transform(
                    matcher,
                    translit(config.normalization.clone()),
                )?
                .and(self.and)?
                .or(self.or)?
                .not(self.not)?
                .build(),
            )
        } else {
            None
        };

        let template = self.template.unwrap_or(if self.gzip {
            "{}.dat.gz".into()
        } else {
            "{}.dat".into()
        });

        if !self.outdir.exists() {
            fs::create_dir_all(&self.outdir)?;
        }

        let mut writer = WriterBuilder::new()
            .gzip(self.gzip)
            .from_path(
                self.outdir
                    .join(template.replace("{}", &chunks.to_string())),
            )?;

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

                        if !filter_set.check(record) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher {
                            if !matcher.is_match(record, &options) {
                                continue;
                            }
                        }

                        if count > 0
                            && count as u32 % self.chunk_size == 0
                        {
                            writer.finish()?;
                            chunks += 1;

                            writer = WriterBuilder::new()
                                .gzip(self.gzip)
                                .from_path(self.outdir.join(
                                    template.replace(
                                        "{}",
                                        &chunks.to_string(),
                                    ),
                                ))?;
                        }

                        writer.write_byte_record(record)?;
                        count += 1;
                    }
                }
            }
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
