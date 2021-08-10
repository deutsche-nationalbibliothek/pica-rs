#[macro_use]
extern crate sophia_api;

use assert_cmd::Command;

use sophia::graph::inmem::FastGraph;
use sophia::graph::Graph;
use sophia::ns::{rdf, Namespace};
use sophia::parser::turtle;
use sophia::term::literal::Literal;
use sophia::triple::stream::TripleSource;

mod skos {
    namespace!(
        "http://www.w3.org/2004/02/skos/core#",
        Concept,
        prefLabel,
        altLabel,
        hiddenLabel,
        broader,
        related
    );
}

pub type StrLiteral = Literal<Box<str>>;

#[test]
fn test_skosify() {
    let mut cmd = Command::cargo_bin("gnd2skos").unwrap();
    let assert = cmd.arg("tests/data/119232022.dat").assert();
    let output = assert.get_output();

    let graph: FastGraph =
        turtle::parse_str(&String::from_utf8(output.stdout.clone()).unwrap())
            .collect_triples()
            .unwrap();

    // check skos:Concept
    let gnd_ns = Namespace::new("http://d-nb.info/gnd/").unwrap();
    let subject = gnd_ns.get("119232022").unwrap();
    assert!(graph
        .contains(&subject, &rdf::type_, &skos::Concept)
        .unwrap());

    // check skos:prefLabel
    let gnd_ns = Namespace::new("http://d-nb.info/gnd/").unwrap();
    let subject = gnd_ns.get("119232022").unwrap();
    assert!(graph
        .contains(
            &subject,
            &skos::prefLabel,
            &StrLiteral::new_lang(
                String::from("Lovelace, Ada King of (1815-1852)"),
                "de",
            )
            .unwrap()
        )
        .unwrap());

    // check skos:altLabel
    let gnd_ns = Namespace::new("http://d-nb.info/gnd/").unwrap();
    let subject = gnd_ns.get("119232022").unwrap();
    assert!(graph
        .contains(
            &subject,
            &skos::altLabel,
            &StrLiteral::new_lang(String::from("Byron Lovelace, Ada"), "de",)
                .unwrap()
        )
        .unwrap());

    // check skos:hiddenLabel
    let gnd_ns = Namespace::new("http://d-nb.info/gnd/").unwrap();
    let subject = gnd_ns.get("119232022").unwrap();
    assert!(graph
        .contains(
            &subject,
            &skos::hiddenLabel,
            &StrLiteral::new_lang(String::from("Lovelace, Ada King of"), "de",)
                .unwrap()
        )
        .unwrap());

    // check skos:related
    let gnd_ns = Namespace::new("http://d-nb.info/gnd/").unwrap();
    let subject = gnd_ns.get("119232022").unwrap();
    let object = gnd_ns.get("118518208").unwrap();

    assert!(graph.contains(&subject, &skos::related, &object).unwrap());
}
