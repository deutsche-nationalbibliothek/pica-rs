#[macro_use]
extern crate clap;

mod cli;
mod skos;
mod utils;
#[macro_use]
pub mod macros;
pub mod event;
pub mod geoplace;
pub mod person;
pub mod topical_term;

use bstr::io::BufReadExt;
use flate2::read::GzDecoder;
use pica::{Filter, Record};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use rdf::graph::Graph;
use rdf::triple::Triple;
use rdf::uri::Uri;
use rdf::writer::{rdf_writer::RdfWriter, turtle_writer::TurtleWriter};

use skos::{Concept, Event, GeoPlace, Person, TopicalTerm};
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

    let mut writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut g = Graph::new(None);
    add_namespace!(g, "xsd", "http://www.w3.org/2001/XMLSchema#");
    add_namespace!(g, "rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
    add_namespace!(g, "skos", "http://www.w3.org/2004/02/skos/core#");
    add_namespace!(g, "dcterms", "http://purl.org/dc/terms/");
    add_namespace!(g, "gnd", "http://d-nb.info/gnd/");

    for result in reader.byte_lines() {
        let line = result?;

        if let Ok(record) = Record::from_bytes(&line) {
            if !filter.matches(&record) {
                continue;
            }

            let bbg = record.first("002@").unwrap().first('0').unwrap();
            let concept: Box<dyn Concept> = match &bbg[..2] {
                "Tp" => Box::new(Person(record)),
                "Ts" => Box::new(TopicalTerm(record)),
                "Tg" => Box::new(GeoPlace(record)),
                "Tf" => Box::new(Event(record)),

                _ => unimplemented!(),
            };

            // skos:Concept
            let sub = g.create_uri_node(&concept.uri());
            let pred = g.create_uri_node(&Uri::new(rdf!("type")));
            let obj = g.create_uri_node(&Uri::new(skos!("Concept")));
            add_triple!(g, &sub, &pred, &obj);

            // dcterms:created
            if let Some((pred, obj)) = concept.created() {
                g.add_triple(&Triple::new(&sub, &pred, &obj));
            }

            // dcterms:modified
            if let Some((pred, obj)) = concept.modified() {
                g.add_triple(&Triple::new(&sub, &pred, &obj));
            }

            // skos:prefLabel
            if let Some((pred, obj)) = concept.pref_label() {
                g.add_triple(&Triple::new(&sub, &pred, &obj));
            }

            // skos:altLabel
            for (pred, obj) in concept.alt_labels() {
                g.add_triple(&Triple::new(&sub, &pred, &obj));
            }

            // skos:hiddenLabel
            for (pred, obj) in concept.hidden_labels() {
                g.add_triple(&Triple::new(&sub, &pred, &obj));
            }

            // break;
        } else if !skip_invalid {
            return Err(CliError::Other(format!(
                "could not read record: {}",
                String::from_utf8(line).unwrap()
            )));
        }
    }

    let ttl_writer = TurtleWriter::new(&g.namespaces());
    writer.write(ttl_writer.write_to_string(&g).unwrap().as_bytes())?;

    Ok(())
}
