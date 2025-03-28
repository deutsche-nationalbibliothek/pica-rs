use std::ffi::OsString;
use std::fs::File;
use std::process::ExitCode;

use clap::{Parser, value_parser};
use hashbrown::HashMap;
use pica_record::prelude::*;
use polars::prelude::*;

use crate::prelude::*;

/// Creates a frequency table of all subfield codes.
#[derive(Parser, Debug)]
pub(crate) struct Describe {
    /// Skip invalid records that can't be decoded
    #[arg(short, long)]
    skip_invalid: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
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

    /// Only examine fields that are specified in the list.
    #[arg(long, short)]
    keep: Option<String>,

    /// Ignore fields that specifield in the list.
    #[arg(long, short)]
    discard: Option<String>,

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

    /// Write output tab-separated (TSV)
    #[arg(long, short, requires = "output")]
    tsv: bool,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Describe {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let discard = parse_predicates(self.discard)?;
        let keep = parse_predicates(self.keep)?;

        let matcher = if let Some(matcher) = self.filter {
            Some(
                RecordMatcherBuilder::with_transform(
                    matcher,
                    translit(config.normalization.clone()),
                )?
                .and(self.and)?
                .not(self.not)?
                .or(self.or)?
                .build(),
            )
        } else {
            None
        };

        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold as f64 / 100f64)
            .case_ignore(self.ignore_case);

        let mut fields: HashMap<String, HashMap<char, usize>> =
            HashMap::new();

        for filename in self.filenames {
            let mut reader =
                ReaderBuilder::new().from_path(filename)?;

            while let Some(mut result) = reader.next_byte_record() {
                match result {
                    Err(e) if e.skip_parse_err(skip_invalid) => {
                        progress.update(true);
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                    Ok(ref mut record) => {
                        progress.update(false);

                        if let Some(ref matcher) = matcher {
                            if !matcher.is_match(record, &options) {
                                continue;
                            }
                        }

                        record.discard(&discard);
                        record.keep(&keep);

                        for field in record.fields() {
                            let tag = field.tag().to_string();

                            let subfields = fields
                                .entry(tag)
                                .or_insert(HashMap::new());

                            for subfield in field.subfields() {
                                subfields
                                    .entry(**subfield.code())
                                    .and_modify(|e| *e += 1)
                                    .or_insert(1);
                            }
                        }
                    }
                }
            }
        }

        let mut codes =
            fields.values().flat_map(|m| m.keys()).collect::<Vec<_>>();
        codes.sort_unstable();
        codes.dedup();

        let tags =
            fields.keys().map(ToString::to_string).collect::<Vec<_>>();

        let mut columns = vec![];
        columns.push(Column::new("field".into(), tags.clone()));

        for code in codes.iter() {
            let mut values = vec![];

            for tag in tags.iter() {
                let counts = fields.get(tag).unwrap();
                let count = counts.get(*code).unwrap_or(&0);
                values.push(*count as u64);
            }

            columns.push(Column::new(code.to_string().into(), values));
        }

        let mut df = DataFrame::new(columns)?
            .lazy()
            .sort(["field"], SortMultipleOptions::default())
            .collect()?;

        match self.output {
            None => {
                let _tmp_env = (
                    tmp_env::set_var(
                        "POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION",
                        "1",
                    ),
                    tmp_env::set_var(
                        "POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES",
                        "1",
                    ),
                    tmp_env::set_var(
                        "POLARS_FMT_TABLE_CELL_NUMERIC_ALIGNMENT",
                        "1",
                    ),
                    tmp_env::set_var("POLARS_FMT_MAX_COLS", "20"),
                    tmp_env::set_var("POLARS_FMT_MAX_ROWS", "50"),
                );

                println!("{df}");
            }
            Some(path)
                if path.to_string_lossy().ends_with(".tsv")
                    || self.tsv =>
            {
                let mut writer = CsvWriter::new(File::create(path)?)
                    .with_separator(b'\t');
                writer.finish(&mut df)?;
            }
            Some(path) if path.to_string_lossy().ends_with(".csv") => {
                let mut writer = CsvWriter::new(File::create(path)?);
                writer.finish(&mut df)?;
            }
            Some(path) => {
                let mut writer = IpcWriter::new(File::create(path)?)
                    .with_compression(Some(IpcCompression::ZSTD));
                writer.finish(&mut df)?;
            }
        }

        progress.finish();

        Ok(ExitCode::SUCCESS)
    }
}
