use pica::{Field, Record};
use sophia::graph::MutableGraph;
use sophia::ns::{rdf, Namespace};
use std::ops::Deref;

use crate::concept::{Concept, StrLiteral};
use crate::ns::skos;

pub struct TopicalTerm<'a>(pub(crate) Record<'a>);

impl<'a> Deref for TopicalTerm<'a> {
    type Target = Record<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TopicalTerm<'a> {
    pub fn get_label(field: &Field) -> Option<StrLiteral> {
        let mut label = String::new();

        if field.exists('a') {
            push_value!(label, field.first('a'));
            push_list!(label, field.all('g'), ", ", " (", ")");
            push_list!(label, field.all('x'), " / ", " / ");
        }

        if !label.is_empty() {
            label = label.replace('@', "");
            Some(StrLiteral::new_lang(label, "de").unwrap())
        } else {
            None
        }
    }
}

impl<'a> Concept for TopicalTerm<'a> {
    fn skosify<G: MutableGraph>(&self, graph: &mut G) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();
        let idn = self.first("003@").unwrap().first('0').unwrap();
        let subj = gnd.get(&idn).unwrap();

        // skos:Concept
        graph.insert(&subj, &rdf::type_, &skos::Concept).unwrap();

        // skos:prefLabel
        for field in self.all("041A") {
            if let Some(label) = Self::get_label(field) {
                graph.insert(&subj, &skos::prefLabel, &label).unwrap();
            }
        }

        // skos:altLabel
        for field in self.all("041@") {
            if let Some(label) = Self::get_label(field) {
                graph.insert(&subj, &skos::altLabel, &label).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sophia::graph::{inmem::FastGraph, Graph};

    #[test]
    fn test_1004916019() {
        let record_str = include_str!("../../../tests/data/1004916019.dat");
        let record = Record::from_bytes(record_str.as_bytes()).unwrap();
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();

        let mut graph = FastGraph::new();

        TopicalTerm(record).skosify(&mut graph);
        assert!(graph
            .contains(
                &gnd.get("1004916019").unwrap(),
                &skos::prefLabel,
                &StrLiteral::new_lang("Plymouth (Marke)", "de").unwrap()
            )
            .unwrap());
    }
}
