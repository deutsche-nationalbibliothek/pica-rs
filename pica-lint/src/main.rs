use clap::Parser;
use pica_record::io::BufReadExt;

use crate::cli::Args;
use crate::formatter::{CsvFormatter, Formatter};
use crate::progress::Progress;
use crate::rules::RuleSet;

mod cli;
mod formatter;
mod lints;
mod progress;
mod rules;
mod stats;
mod util;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut rulesets = vec![];
    let mut formatter: Box<dyn Formatter> =
        Box::new(CsvFormatter::new(args.output));

    for path in args.rules.iter() {
        rulesets.push(RuleSet::from_path(path)?);
    }

    let mut progress = Progress::new(
        rulesets.iter().map(|rs| rs.name.to_string()).collect(),
    );

    for path in args.filenames {
        let mut reader = util::reader(path)?;
        reader.for_pica_record(|result| {
            if let Ok(record) = result {
                for rs in rulesets.iter_mut() {
                    rs.preprocess(&record);

                    let result = rs.check(&record, &mut formatter);
                    progress.update_stats(&rs.name, &result);
                }
            }

            progress.update();
            Ok(true)
        })?;
    }

    for rs in rulesets.iter_mut() {
        let result = rs.finish(&mut formatter);
        progress.update_stats(&rs.name, &result);
        progress.update();
    }

    formatter.finish()?;
    progress.finish();

    Ok(())
}
