use crate::cmds::Config;
use crate::util::{App, CliArgs, CliError, CliResult};
use bstr::io::BufReadExt;
use bstr::ByteSlice;
use clap::Arg;
use pica::{self, ByteRecord};

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::create_dir;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

pub fn cli() -> App {
    App::new("partition")
        .about("Partition a list of records by subfield value.")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .about("skip invalid records"),
        )
        .arg(
            Arg::new("outdir")
                .short('o')
                .long("--outdir")
                .value_name("outdir")
                .default_value("."),
        )
        .arg(
            Arg::new("template")
                .short('t')
                .long("--template")
                .value_name("template")
                .default_value("{}.dat"),
        )
        .arg(Arg::new("path").required(true))
        .arg(Arg::new("filename"))
}

pub fn run(args: &CliArgs) -> CliResult<()> {
    let ctx = Config::new();
    let filename_template = args.value_of("template").unwrap_or("{}.dat");
    let skip_invalid = args.is_present("skip-invalid");
    let reader = ctx.reader(args.value_of("filename"))?;

    let outdir = Path::new(args.value_of("outdir").unwrap());
    if !outdir.exists() {
        create_dir(outdir)?;
    }

    let mut writers: HashMap<Vec<u8>, Box<dyn Write + 'static>> =
        HashMap::new();
    let path = pica::Path::from_str(args.value_of("path").unwrap())?;

    for result in reader.byte_lines() {
        let line = result?;

        if let Ok(record) = ByteRecord::from_bytes(line.clone()) {
            for value in record.path(&path) {
                let mut entry = writers.entry(value.as_bytes().to_vec());
                let writer = match entry {
                    Entry::Vacant(vacant) => {
                        let value = String::from_utf8(value.to_vec()).unwrap();
                        let writer = ctx.writer(
                            outdir
                                .join(filename_template.replace("{}", &value))
                                .to_str(),
                        )?;

                        vacant.insert(writer)
                    }
                    Entry::Occupied(ref mut occupied) => occupied.get_mut(),
                };

                writer.write_all(&line)?;
                writer.write_all(b"\n")?;
            }
        } else if !skip_invalid {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                String::from_utf8(line).unwrap()
            )));
        }
    }

    Ok(())
}
