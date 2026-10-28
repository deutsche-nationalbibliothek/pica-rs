use std::ffi::OsString;
use std::fs::File;
use std::io::{Write, stdout};
use std::process::ExitCode;

use clap::Parser;
use hashbrown::HashMap;
use pica_record::prelude::*;

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

        let inner: Box<dyn Write> = match self.output {
            None => Box::new(stdout().lock()),
            Some(ref path) => Box::new(File::create(path)?),
        };

        let delimiter = match self.output {
            Some(ref path)
                if path.to_string_lossy().ends_with(".tsv") =>
            {
                b'\t'
            }
            _ => b',',
        };

        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(inner);

        wtr.write_field("field")?;
        for code in codes.iter() {
            wtr.write_field(format!("{code}"))?;
        }

        wtr.write_record(None::<&[u8]>)?;

        for tag in tags.iter() {
            let mut values = vec![];

            values.push(tag.to_string());

            for code in codes.iter() {
                let counts = fields.get(tag).unwrap();
                let count = counts.get(*code).unwrap_or(&0);
                values.push(count.to_string());
            }

            wtr.write_record(values)?;
        }

        progress.finish();
        Ok(ExitCode::SUCCESS)
    }
}
