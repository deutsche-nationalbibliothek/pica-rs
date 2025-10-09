use std::ffi::OsString;
use std::fs::File;
use std::process::ExitCode;

use clap::Parser;
use hashbrown::HashMap;
use pica_record::prelude::*;
use polars::prelude::*;

use crate::prelude::*;

/// Creates a frequency table of all subfield codes.
#[derive(Parser, Debug)]
pub(crate) struct Describe {
    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// Only examine fields that are specified in the list.
    #[arg(long, short)]
    keep: Option<String>,

    /// Ignore fields that specifield in the list.
    #[arg(long, short)]
    discard: Option<String>,

    /// Write output comma-separated (CSV)
    #[arg(long, short, requires = "output", conflicts_with = "tsv")]
    csv: bool,

    /// Write output tab-separated (TSV)
    #[arg(long, short, requires = "output", conflicts_with = "csv")]
    tsv: bool,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    #[command(flatten, next_help_heading = "Filter options")]
    pub(crate) filter_opts: FilterOpts,
}

impl Describe {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut count = 0;

        let discard = parse_predicates(self.discard)?;
        let keep = parse_predicates(self.keep)?;

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let options = MatcherOptions::from(&self.filter_opts);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;

        let mut fields: HashMap<String, HashMap<char, usize>> =
            HashMap::new();

        'outer: for filename in self.filenames {
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

                        if !filter_set.check(record) {
                            continue;
                        }

                        if let Some(ref matcher) = matcher
                            && !matcher.is_match(record, &options)
                        {
                            continue;
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
            Some(path)
                if path.to_string_lossy().ends_with(".csv")
                    || self.csv =>
            {
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
