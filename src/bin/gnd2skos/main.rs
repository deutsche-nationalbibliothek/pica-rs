#[macro_use]
extern crate clap;

#[macro_use]
extern crate sophia_api;

mod cli;
mod concept;
mod corporate_body;
mod event;
mod geoplace;
#[macro_use]
mod macros;
mod ns;
mod person;
mod topical_term;

mod utils;
mod work;

use pica::{Filter, ReaderBuilder};
use std::fs::File;
use std::io::{self, Write};

use sophia::graph::inmem::LightGraph;
use sophia::serializer::nt::NtSerializer;
use sophia::serializer::*;

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

    let mut writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut g = LightGraph::new();

    let mut reader = ReaderBuilder::new()
        .skip_invalid(args.is_present("skip-invalid"))
        .from_path_or_stdin(args.value_of("filename"))?;

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

    for result in reader.records() {
        let record = result?;

        if !filter.matches(&record) {
            continue;
        }

        let bbg = record.first("002@").unwrap().first('0').unwrap();

        match &bbg[..2] {
            b"Tb" => CorporateBody(record).skosify(&mut g, &args),
            b"Tf" => Event(record).skosify(&mut g, &args),
            b"Tg" => GeoPlace(record).skosify(&mut g, &args),
            b"Tp" => Person(record).skosify(&mut g, &args),
            b"Ts" => TopicalTerm(record).skosify(&mut g, &args),
            b"Tu" => Work(record).skosify(&mut g, &args),
            _ => unimplemented!(),
        }
    }

    let mut nt_stringifier = NtSerializer::new_stringifier();
    writer.write_all(
        nt_stringifier
            .serialize_graph(&g)
            .unwrap()
            .as_str()
            .as_bytes(),
    )?;

    Ok(())
}
