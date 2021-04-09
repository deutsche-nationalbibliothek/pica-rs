use crate::util::{App, CliArgs, CliResult};
use bstr::BString;
use clap::Arg;
use pica::{Path, ReaderBuilder};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::str::FromStr;

pub fn cli() -> App {
    App::new("frequency")
        .about("Compute a frequency table of a subfield.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("--limit")
                .value_name("n")
                .about("Limit the result to the <n> most common items.")
                .default_value("0"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .about("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("path").required(true))
        .arg(Arg::new("filename"))
}

fn writer(filename: Option<&str>) -> CliResult<Box<dyn Write>> {
    Ok(match filename {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    })
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let limit: u64 = args.value_of("limit").unwrap().parse().unwrap();

    let mut writer =
        csv::WriterBuilder::new().from_writer(writer(args.value_of("output"))?);

    let mut reader = ReaderBuilder::new()
        .skip_invalid(args.is_present("skip-invalid"))
        .from_path_or_stdin(args.value_of("filename"))?;

    let mut ftable: HashMap<BString, u64> = HashMap::new();
    let path = Path::from_str(args.value_of("path").unwrap())?;

    for result in reader.records() {
        let record = result?;

        for value in record.path(&path) {
            *ftable.entry(value.to_owned()).or_insert(0) += 1;
        }
    }

    let mut ftable_sorted: Vec<(&BString, &u64)> = ftable.iter().collect();
    ftable_sorted.sort_by(|a, b| b.1.cmp(a.1));

    for (value, frequency) in ftable_sorted {
        if *frequency < limit {
            break;
        }
        writer.write_record(&[value, &BString::from(frequency.to_string())])?;
    }

    writer.flush()?;
    Ok(())
}
