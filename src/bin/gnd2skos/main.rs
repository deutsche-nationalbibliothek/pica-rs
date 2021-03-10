#[macro_use]
extern crate clap;

mod cli;
mod concept;
mod corporate_body;
mod event;
mod geoplace;
mod person;
mod topical_term;
mod utils;
mod work;

use bstr::io::BufReadExt;
use flate2::read::GzDecoder;
use pica::{Filter, Record};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use sophia::graph::{inmem::FastGraph, inmem::LightGraph, *};
use sophia::ns::Namespace;
use sophia::ns::{rdf, xsd};
use sophia::serializer::{nt::NtSerializer, *};
use sophia::term::iri::Iri;
use sophia::term::literal::Literal;

use concept::Concept;
use corporate_body::CorporateBody;
use event::Event;
use geoplace::GeoPlace;
use person::Person;
use topical_term::TopicalTerm;
use utils::{CliError, CliResult};
use work::Work;

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

    let mut g = FastGraph::new();
    let skos = Namespace::new("http://www.w3.org/2004/02/skos/core#").unwrap();
    let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();

    for result in reader.byte_lines() {
        let line = result?;

        if let Ok(record) = Record::from_bytes(&line) {
            if !filter.matches(&record) {
                continue;
            }

            let bbg = record.first("002@").unwrap().first('0').unwrap();
            let concept: Box<dyn Concept> = match &bbg[..2] {
                "Tb" => Box::new(CorporateBody(record)),
                "Tf" => Box::new(Event(record)),
                "Tg" => Box::new(GeoPlace(record)),
                "Tp" => Box::new(Person(record)),
                "Ts" => Box::new(TopicalTerm(record)),
                "Tu" => Box::new(Work(record)),
                _ => unimplemented!(),
            };

            g.insert(
                &gnd.get(&concept.idn()).unwrap(),
                &rdf::type_,
                &skos.get("Concept").unwrap(),
            )
            .unwrap();

            // skos:prefLabel
            if let Some(pref_label) = concept.pref_label() {
                g.insert(
                    &gnd.get(&concept.idn()).unwrap(),
                    &skos.get("prefLabel").unwrap(),
                    &Literal::<Box<str>>::new_dt(
                        pref_label,
                        Iri::<&'static str>::from(xsd::string),
                    ),
                )
                .unwrap();
            }

            // skos:altLabel
            for alt_label in concept.alt_labels() {
                g.insert(
                    &gnd.get(&concept.idn()).unwrap(),
                    &skos.get("altLabel").unwrap(),
                    &Literal::<Box<str>>::new_dt(
                        alt_label,
                        Iri::<&'static str>::from(xsd::string),
                    ),
                )
                .unwrap();
            }

            // skos:hiddenLabel
            for hidden_label in concept.hidden_labels() {
                g.insert(
                    &gnd.get(&concept.idn()).unwrap(),
                    &skos.get("altLabel").unwrap(),
                    &Literal::<Box<str>>::new_dt(
                        hidden_label,
                        Iri::<&'static str>::from(xsd::string),
                    ),
                )
                .unwrap();
            }

            // break;
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
