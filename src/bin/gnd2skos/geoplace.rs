use pica::{Field, Record};
use sophia::graph::MutableGraph;
use sophia::ns::{rdf, Namespace};
use std::ops::Deref;

use crate::concept::{Concept, StrLiteral};
use crate::ns::skos;

pub struct GeoPlace<'a>(pub(crate) Record<'a>);

impl<'a> Deref for GeoPlace<'a> {
    type Target = Record<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> GeoPlace<'a> {
    pub fn get_label(field: &Field) -> Option<StrLiteral> {
        let mut label = String::new();

        for subfield in field.iter() {
            let value = String::from_utf8(subfield.value().to_vec()).unwrap();

            match subfield.code() {
                'a' => {
                    label.push_str(&value.replace('@', ""));
                }
                'g' | 'z' => label.push_str(&format!(" ({})", value)),
                'x' => label.push_str(&format!(" / {}", value)),
                _ => continue,
            }
        }

        if !label.is_empty() {
            Some(StrLiteral::new_lang(label, "de").unwrap())
        } else {
            None
        }
    }
}

impl<'a> Concept for GeoPlace<'a> {
    fn skosify<G: MutableGraph>(&self, graph: &mut G) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();
        let idn = self.first("003@").unwrap().first('0').unwrap();
        let subj = gnd.get(&idn).unwrap();

        // skos:Concept
        graph.insert(&subj, &rdf::type_, &skos::Concept).unwrap();

        // skos:prefLabel
        if let Some(label) = Self::get_label(self.first("065A").unwrap()) {
            graph.insert(&subj, &skos::prefLabel, &label).unwrap();
        }

        // skos:altLabel
        for field in self.all("065@") {
            if let Some(label) = Self::get_label(field) {
                graph.insert(&subj, &skos::altLabel, &label).unwrap();
            }
        }
    }
}
