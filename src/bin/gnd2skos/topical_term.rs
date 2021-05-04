use pica::{Field, StringRecord};
use sophia::graph::MutableGraph;
use sophia::ns::{rdf, Namespace};
use std::ops::Deref;

use bstr::ByteSlice;

use crate::concept::{Concept, StrLiteral};
use crate::ns::skos;

pub struct TopicalTerm(pub(crate) StringRecord);

impl Deref for TopicalTerm {
    type Target = StringRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TopicalTerm {
    pub fn get_label(field: &Field) -> Option<StrLiteral> {
        let mut label = String::new();

        if field.contains_code('a') {
            push_value!(label, field.first('a'));
            push_list!(
                label,
                field.all('g').unwrap_or_default(),
                ", ",
                " (",
                ")"
            );
            push_list!(label, field.all('x').unwrap_or_default(), " / ", " / ");
        }

        if !label.is_empty() {
            label = label.replace('@', "");
            Some(StrLiteral::new_lang(label, "de").unwrap())
        } else {
            None
        }
    }
}

impl Concept for TopicalTerm {
    fn skosify<G: MutableGraph>(&self, graph: &mut G) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();
        let idn = self.first("003@").unwrap().first('0').unwrap();
        let subj = gnd.get(idn.to_str().unwrap()).unwrap();

        // skos:Concept
        graph.insert(&subj, &rdf::type_, &skos::Concept).unwrap();

        // skos:prefLabel
        for field in self.all("041A").unwrap_or_default() {
            if let Some(label) = Self::get_label(field) {
                graph.insert(&subj, &skos::prefLabel, &label).unwrap();
            }
        }

        // skos:altLabel
        for field in self.all("041@").unwrap_or_default() {
            if let Some(label) = Self::get_label(field) {
                graph.insert(&subj, &skos::altLabel, &label).unwrap();
            }
        }
    }
}
