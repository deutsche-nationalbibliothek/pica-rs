use std::ffi::OsString;
use std::process::ExitCode;

use clap::Parser;
use pica_record::prelude::*;

use crate::prelude::*;

/// Return records within a range
///
/// The slice command returns records within a range. The range starts
/// at position 0 and is specified as an half-open interval, which means
/// that the end-position is not included.
///
/// Note: A slice might have less records than specified, if there are
/// not enough records to read or a record within a range is invalid.
#[derive(Parser, Debug)]
pub(crate) struct Slice {
    /// The start position of the slice
    ///
    /// If no start position is specified, then the slice starts from
    /// the first record at position 0.
    #[arg(long, default_value = "0", hide_default_value = true)]
    start: usize,

    /// The end position of the slice
    ///
    /// This option specifies the end position of the slice, which
    /// isn't included in the output. If no end position is specified,
    /// the slice continues to the last record. The resulting slice
    /// may contain less records if invalid records are skipped.
    ///
    /// This options can't be combined with the `length` option.
    #[arg(
        long,
        default_value = "0",
        conflicts_with = "length",
        hide_default_value = true
    )]
    end: usize,

    /// The length of the slice
    ///
    /// This options specifies the maximum number of (valid) records
    /// read from the start position.
    ///
    /// This options can't be combined with the `end` option.
    #[arg(
        long,
        default_value = "0",
        conflicts_with = "end",
        hide_default_value = true
    )]
    length: usize,

    /// Compress output in gzip format
    #[arg(long, short)]
    gzip: bool,

    /// Append to the given file, do not overwrite
    #[arg(long, short)]
    append: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format
    ///
    /// If no filenames where given or a filename is "-", data is read
    /// from standard input (stdin).
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    #[command(flatten, next_help_heading = "Filter options")]
    pub(crate) filter_opts: FilterOpts,
}

impl Slice {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut count = 0;

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let options = MatcherOptions::from(&self.filter_opts);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;

        let mut writer = WriterBuilder::new()
            .gzip(self.gzip)
            .append(self.append)
            .from_path_or_stdout(self.output)?;

        let mut range = if self.end > 0 {
            self.start..self.end
        } else if self.length > 0 {
            self.start..(self.start + self.length)
        } else {
            self.start..usize::MAX
        };

        let mut progress = Progress::new(self.progress);
        let mut i = 0;

        'outer: for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(result) = reader.next_byte_record() {
                match result {
                    Err(e) if e.skip_parse_err(skip_invalid) => {
                        progress.update(true);
                        if self.length > 0 && range.end < usize::MAX {
                            range.end += 1;
                        }
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                    Ok(ref record) => {
                        progress.update(false);

                        if !filter_set.check(&record) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher
                            && !matcher.is_match(&record, &options)
                        {
                            continue;
                        }

                        if range.contains(&i) {
                            writer.write_byte_record(record)?;
                        } else if i < range.start {
                            i += 1;
                            continue;
                        } else {
                            break 'outer;
                        }

                        count += 1;
                        if self.filter_opts.limit > 0
                            && count >= self.filter_opts.limit
                        {
                            break 'outer;
                        }
                    }
                }

                i += 1;
            }
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
