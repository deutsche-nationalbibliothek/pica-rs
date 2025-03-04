use std::ffi::OsString;
use std::process::ExitCode;

use pica_record::prelude::*;
use set::RuleSet;
use writer::writer;

use crate::prelude::*;

mod checks;
mod rule;
mod set;
mod writer;

/// Count records, fields and subfields
#[derive(clap::Parser, Debug)]
pub(crate) struct Check {
    /// Whether to skip invalid records or not
    #[arg(short, long)]
    skip_invalid: bool,

    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// A set of rules to be checked.
    #[arg(long = "rule-set", short = 'R', required = true)]
    rules: Vec<OsString>,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Check {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid = self.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut writer = writer(self.output)?;

        let mut rulesets = self
            .rules
            .iter()
            .map(RuleSet::from_path)
            .collect::<Result<Vec<_>, _>>()?;

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
                        for rs in rulesets.iter_mut() {
                            rs.preprocess(record);
                            rs.check(record, &mut writer)?;
                        }

                        progress.update(false);
                    }
                }
            }
        }

        progress.finish();

        Ok(ExitCode::SUCCESS)
    }
}
