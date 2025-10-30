use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use pica_record::prelude::*;
use regex::RegexSetBuilder;
use set::RuleSet;
use writer::Writer;

use crate::prelude::*;

mod checks;
mod rule;
mod set;
pub(crate) mod writer;

/// Checks records against rule sets.
#[derive(clap::Parser, Debug)]
pub(crate) struct Check {
    /// Show progress bar (requires `-o`/`--output`).
    #[arg(short, long, requires = "output")]
    progress: bool,

    /// A set of rules to be checked.
    #[arg(long = "rule-set", short = 'R', required = true)]
    rules: Vec<OsString>,

    /// A list of patterns to restrict the rule set to selected rules.
    #[arg(long = "rule", short = 'r')]
    filter: Vec<String>,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<PathBuf>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,

    #[command(flatten, next_help_heading = "Filter options")]
    filter_opts: FilterOpts,
}

impl Check {
    pub(crate) fn execute(self, config: &Config) -> CliResult {
        let skip_invalid =
            self.filter_opts.skip_invalid || config.skip_invalid;
        let mut progress = Progress::new(self.progress);
        let mut writer = Writer::from_path(self.output)?;
        let mut count = 0;

        let filter_set = FilterSet::try_from(&self.filter_opts)?;
        let options = MatcherOptions::from(&self.filter_opts);
        let matcher = self
            .filter_opts
            .matcher(config.normalization.clone(), None)?;

        let mut rulesets = self
            .rules
            .iter()
            .map(|path| {
                RuleSet::new(path, config.normalization.as_ref())
            })
            .collect::<Result<Vec<_>, _>>()?;

        if !self.filter.is_empty() {
            let re =
                RegexSetBuilder::new(self.filter).build().map_err(
                    |_| CliError::Other("invalid rule filter".into()),
                )?;

            for rs in rulesets.iter_mut() {
                rs.rules.retain(|k, _| re.is_match(k));
            }
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
                            && !matcher.is_match(record, &options)
                        {
                            continue;
                        }

                        for rs in rulesets.iter_mut() {
                            rs.preprocess(record);
                            rs.check(record, &mut writer)?;
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

        for rs in rulesets.iter_mut() {
            rs.finish(&mut writer)?;
        }

        progress.finish();
        writer.finish()?;

        Ok(ExitCode::SUCCESS)
    }
}
