use clap::ArgMatches;
use pica::{Field, StringRecord};
use sophia::graph::MutableGraph;
use sophia::ns::{rdf, Namespace};
use std::ops::Deref;

use bstr::ByteSlice;

use crate::concept::{Concept, StrLiteral};
use crate::ns::skos;

pub struct Event(pub(crate) StringRecord);

const CHECK: [char; 4] = ['n', 'd', 'c', 'g'];

impl Deref for Event {
    type Target = StringRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Event {
    pub fn get_label(field: &Field) -> Option<StrLiteral> {
        let mut parens = String::new();
        let mut label = String::new();

        for subfield in field.iter() {
            let value = String::from_utf8(subfield.value().to_vec()).unwrap();

            if !CHECK.contains(&subfield.code()) && !parens.is_empty() {
                label.push_str(&format!(" ({})", parens));
                parens.clear();
            }

            match subfield.code() {
                'a' => {
                    label.push_str(&value.replace('@', ""));
                }
                'x' | 'b' => {
                    label.push_str(&format!(" / {}", value));
                }
                'g' => {
                    if parens.is_empty() {
                        label.push_str(&format!(" ({})", value))
                    } else {
                        parens.push_str(&format!(" ({})", value))
                    }
                }
                'n' | 'd' | 'c' => {
                    if !parens.is_empty() {
                        parens.push_str(", ");
                    }
                    parens.push_str(&value);
                }
                _ => continue,
            }
        }

        if !parens.is_empty() {
            label.push_str(&format!(" ({})", parens));
        }

        if !label.is_empty() {
            Some(StrLiteral::new_lang(label, "de").unwrap())
        } else {
            None
        }
    }
}

impl Concept for Event {
    fn skosify<G: MutableGraph>(&self, graph: &mut G, args: &ArgMatches) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();
        let idn = self.first("003@").unwrap().first('0').unwrap();
        let subj = gnd.get(idn.to_str().unwrap()).unwrap();

        // skos:Concept
        graph.insert(&subj, &rdf::type_, &skos::Concept).unwrap();

        // skos:prefLabel
        if let Some(label) = Self::get_label(self.first("030A").unwrap()) {
            graph.insert(&subj, &skos::prefLabel, &label).unwrap();
        }

        // skos:altLabel
        for field in self.all("030@").unwrap_or_default() {
            if let Some(label) = Self::get_label(field) {
                graph.insert(&subj, &skos::altLabel, &label).unwrap();
            }
        }

        // skos:broader or skos:related
        for field in ["022R", "028R", "029R", "030R", "041R", "065R"] {
            self.add_relations(&subj, self.all(field), graph, args);
        }
    }
}
