use crate::commands::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use clap::{Arg, SubCommand};
use pica::Record;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::create_dir;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;

pub fn cli() -> App {
    SubCommand::with_name("partition")
        .about("Partition a list of records by subfield value.")
        .arg(
            Arg::with_name("skip-invalid")
                .short("s")
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::with_name("outdir")
                .short("o")
                .long("--outdir")
                .value_name("outdir")
                .default_value("."),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("--template")
                .value_name("template")
                .default_value("{}.dat"),
        )
        .arg(Arg::with_name("path").required(true))
        .arg(Arg::with_name("filenames").multiple(true).required(true))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let config = Config::new();
    let filename_template = args.value_of("filename").unwrap_or("{}.dat");
    let skip_invalid = args.is_present("skip-invalid");
    let path_str = args.value_of("path").unwrap();
    let path = path_str.parse::<pica::Path>().unwrap();

    let outdir = Path::new(args.value_of("outdir").unwrap());
    if !outdir.exists() {
        create_dir(outdir)?;
    }

    let mut writers: HashMap<Vec<u8>, Box<dyn Write + 'static>> =
        HashMap::new();

    for filename in args.values_of("filenames").unwrap() {
        let reader = config.reader(Some(filename))?;

        for line in reader.lines() {
            let line = line.unwrap();
            if let Ok(record) = Record::decode(&line) {
                for value in record.path(&path) {
                    let mut entry = writers.entry(value.as_bytes().to_vec());
                    let writer = match entry {
                        Entry::Occupied(ref mut occupied) => occupied.get_mut(),
                        Entry::Vacant(vacant) => {
                            let writer = config.writer(
                                outdir
                                    .join(
                                        filename_template.replace("{}", &value),
                                    )
                                    .to_str(),
                            )?;

                            vacant.insert(writer)
                        }
                    };

                    writer.write_all(line.as_bytes())?;
                    writer.write_all(b"\n")?;
                }
            } else if !skip_invalid {
                return Err(CliError::Other(format!(
                    "could not read record: {}",
                    line
                )));
            }
        }
    }

    Ok(())
}
