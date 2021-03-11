#[macro_use]
extern crate clap;

#[macro_use]
extern crate sophia_api;

mod cli;
#[macro_use]
mod concept;
mod corporate_body;
// mod event;
// mod geoplace;
mod person;
mod topical_term;
mod utils;
// mod work;
mod ns;

use bstr::io::BufReadExt;
use flate2::read::GzDecoder;
use pica::{Filter, Record};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use sophia::graph::inmem::LightGraph;
// use sophia::ns::rdf;
// use sophia::ns::Namespace;
use sophia::serializer::{nt::NtSerializer, *};
// use sophia::term::literal::Literal;

use concept::Concept;
use corporate_body::CorporateBody;
// use event::Event;
// use geoplace::GeoPlace;
use person::Person;
use topical_term::TopicalTerm;
use utils::{CliError, CliResult};
// use work::Work;

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

    let mut writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut g = LightGraph::new();

    for result in reader.byte_lines() {
        let line = result?;

        if let Ok(record) = Record::from_bytes(&line) {
            if !filter.matches(&record) {
                continue;
            }

            let bbg = record.first("002@").unwrap().first('0').unwrap();

            match &bbg[..2] {
                "Tb" => CorporateBody(record).skosify(&mut g),
                // "Tf" => Event(record).skosify(&mut g),
                // "Tg" => GeoPlace(record).skosify(&mut g),
                "Tp" => Person(record).skosify(&mut g),
                "Ts" => TopicalTerm(record).skosify(&mut g),
                // "Tu" => Work(record).skosify(&mut g),
                _ => unimplemented!(),
            }

            break;
        } else if !skip_invalid {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                String::from_utf8(line).unwrap()
            )));
        }
    }

    let mut nt_stringifier = NtSerializer::new_stringifier();
    writer.write(
        nt_stringifier
            .serialize_graph(&mut g)
            .unwrap()
            .as_str()
            .as_bytes(),
    )?;

    Ok(())
}
