use clap::Parser;
use pica_record::io::BufReadExt;

use crate::cli::Args;
use crate::formatter::{CsvFormatter, Formatter};
use crate::progress::Progress;
use crate::rules::Manifest;

mod cli;
mod formatter;
mod lints;
mod progress;
mod rules;
mod util;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut formatter: Box<dyn Formatter> =
        Box::new(CsvFormatter::new(args.output));
    let mut rulesets = vec![];

    for path in args.rules.iter() {
        rulesets.push(Manifest::from_path(path)?);
    }

    let mut progress = Progress::new(
        rulesets.iter().map(|r| r.id.to_string()).collect(),
    );

    for path in args.filenames {
        let mut reader = util::reader(path)?;
        reader.for_pica_record(|result| {
            if let Ok(record) = result {
                for rs in rulesets.iter() {
                    if let Some(ref scope) = rs.scope {
                        if !scope.is_match(&record, &Default::default())
                        {
                            continue;
                        }
                    }

                    let (errors, warnings, infos) =
                        rs.check(&record, &mut formatter).unwrap();

                    progress.update_stats(
                        &rs.id,
                        rs.rules.len(),
                        errors,
                        warnings,
                        infos,
                    );
                }

                progress.update();
            }

            Ok(true)
        })?;
    }

    formatter.finish()?;
    progress.finish();

    Ok(())
}
