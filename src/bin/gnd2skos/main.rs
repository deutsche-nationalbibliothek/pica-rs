#[macro_use]
extern crate clap;

mod cli;
mod utils;

use bstr::io::BufReadExt;
use flate2::read::GzDecoder;
use pica::{Filter, Record};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use utils::{CliError, CliResult};

fn main() -> CliResult<()> {
    let args = cli::build_cli().get_matches();
    let skip_invalid = args.is_present("skip-invalid");

    let filter = match args.value_of("filter") {
        None => Filter::True,
        Some(filter_str) => match Filter::decode(filter_str) {
            Ok(filter) => filter,
            Err(_) => {
                return Err(CliError::Other(format!(
                    "invalid filter: {}",
                    filter_str
                )))
            }
        },
    };

    let reader: Box<dyn BufRead> = match args.value_of("filename") {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => {
            let path = Path::new(filename);

            let reader: Box<dyn Read> =
                if path.extension() == Some(OsStr::new("gz")) {
                    Box::new(GzDecoder::new(File::open(path)?))
                } else {
                    Box::new(File::open(path)?)
                };

            Box::new(BufReader::new(reader))
        }
    };

    let writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    for result in reader.byte_lines() {
        let line = result?;

        if let Ok(_record) = Record::from_bytes(&line) {
            println!("record = {:?}", _record);
            break;
        } else if !skip_invalid {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                String::from_utf8(line).unwrap()
            )));
        }
    }

    Ok(())
}
