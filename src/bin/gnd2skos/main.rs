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
mod ignore_list;
mod ns;
mod person;
mod topical_term;

mod utils;
mod work;

use clap::ArgMatches;
use ignore_list::IgnoreList;
use pica::{MatcherFlags, ReaderBuilder, RecordMatcher};
use std::fs::File;
use std::io::{self, Write};
use std::str::FromStr;

use sophia::graph::inmem::LightGraph;
use sophia::serializer::nt::NtSerializer;

use concept::Concept;
use corporate_body::CorporateBody;
use event::Event;
use geoplace::GeoPlace;
use person::Person;
use sophia_api::serializer::{Stringifier, TripleSerializer};
use topical_term::TopicalTerm;
use utils::{CliError, CliResult};
use work::Work;

#[derive(Debug)]
pub struct AppContext<'a> {
    args: &'a ArgMatches,
    label_ignore_list: &'a IgnoreList,
}

fn main() -> CliResult<()> {
    let args = cli::build_cli().get_matches();
    let ignore_case = args.is_present("ignore-case");

    let label_ignore_list = match args.value_of("label-ignore-list") {
        Some(filename) => IgnoreList::from_path(filename)?,
        None => IgnoreList::default(),
    };

    let ctx = AppContext {
        args: &args,
        label_ignore_list: &label_ignore_list,
    };

    let mut writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut g = LightGraph::new();

    let mut reader = ReaderBuilder::new()
        .skip_invalid(args.is_present("skip-invalid"))
        .from_path_or_stdin(args.value_of("filename"))?;

    let matcher = match args.value_of("filter") {
        None => RecordMatcher::True,
        Some(filter_str) => match RecordMatcher::from_str(filter_str) {
            Ok(matcher) => matcher,
            Err(_) => {
                return Err(CliError::Other(format!(
                    "invalid filter: {}",
                    filter_str
                )))
            }
        },
    };

    let flags = MatcherFlags { ignore_case };

    for result in reader.records() {
        let record = result?;

        if !matcher.is_match(&record, &flags) {
            continue;
        }

        let bbg = record.first("002@").unwrap().first('0').unwrap();

        match &bbg[..2] {
            b"Tb" => CorporateBody(record).skosify(&mut g, &ctx),
            b"Tf" => Event(record).skosify(&mut g, &ctx),
            b"Tg" => GeoPlace(record).skosify(&mut g, &ctx),
            b"Tp" => Person(record).skosify(&mut g, &ctx),
            b"Ts" => TopicalTerm(record).skosify(&mut g, &ctx),
            b"Tu" => Work(record).skosify(&mut g, &ctx),
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
