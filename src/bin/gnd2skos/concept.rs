use crate::ns::skos;
use bstr::ByteSlice;
use clap::ArgMatches;
use pica::Field;
use sophia::graph::MutableGraph;
use sophia::ns::Namespace;
use sophia::term::literal::Literal;
use sophia::term::SimpleIri;

pub type StrLiteral = Literal<Box<str>>;

pub trait Concept {
    fn skosify<G: MutableGraph>(&self, graph: &mut G, args: &ArgMatches);

    fn add_relations<G: MutableGraph>(
        &self,
        subj: &SimpleIri,
        fields: Option<Vec<&Field>>,
        graph: &mut G,
        args: &ArgMatches,
    ) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();

        for field in fields.unwrap_or_default() {
            if !field.contains_code('9') || !field.contains_code('4') {
                continue;
            }

            if let Some(code) = field.first('4') {
                let gnd_id = gnd
                    .get(field.first('9').unwrap().to_str().unwrap())
                    .unwrap();

                if code.starts_with(b"ob") && !args.is_present("no-broader") {
                    graph.insert(subj, &skos::broader, &gnd_id).unwrap();
                } else if !args.is_present("no-related") {
                    graph.insert(subj, &skos::related, &gnd_id).unwrap();
                }
            }
        }
    }
}
