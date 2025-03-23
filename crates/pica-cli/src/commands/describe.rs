use std::ffi::OsString;
use std::process::ExitCode;

use clap::{Parser, value_parser};
use comfy_table::Table;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use hashbrown::HashMap;
use pica_record::prelude::*;

use crate::prelude::*;

/// Creates a summary statistics of (sub-)fields.
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

        let mut header = vec!["Field".to_string()];
        header.extend(codes.iter().map(ToString::to_string));

        let mut tags = fields.keys().collect::<Vec<_>>();
        tags.sort_unstable();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_width(72)
            .set_header(header);

        for tag in tags {
            let counts = fields.get(tag).unwrap();
            let mut row = vec![tag.to_string()];

            for code in codes.iter() {
                let count = counts.get(*code).unwrap_or(&0);
                row.push(count.to_string());
            }

            table.add_row(row);
        }

        progress.finish();
        println!("{table}");

        Ok(ExitCode::SUCCESS)
    }
}
