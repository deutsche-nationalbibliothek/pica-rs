use clap::ArgMatches;
use pica::{Field, StringRecord};
use sophia::graph::MutableGraph;
use sophia::ns::{rdf, Namespace};
use std::ops::Deref;

use bstr::ByteSlice;

use crate::concept::{Concept, StrLiteral};
use crate::ns::skos;

pub struct CorporateBody(pub(crate) StringRecord);

impl Deref for CorporateBody {
    type Target = StringRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CorporateBody {
    pub fn get_label(field: &Field) -> Option<StrLiteral> {
        let mut label = String::new();

        for subfield in field.iter() {
            let value = String::from_utf8(subfield.value().to_vec()).unwrap();

            match subfield.code() {
                'a' => {
                    label.push_str(&value.replace('@', ""));
                }
                'g' => {
                    label.push_str(&format!(" ({})", value));
                }
                'x' | 'b' => {
                    label.push_str(&format!(" / {}", value));
                }
                'n' => {
                    label.push_str(&format!(", {}", value));
                }
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

impl Concept for CorporateBody {
    fn skosify<G: MutableGraph>(&self, graph: &mut G, args: &ArgMatches) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();
        let idn = self.first("003@").unwrap().first('0').unwrap();
        let subj = gnd.get(idn.to_str().unwrap()).unwrap();

        // skos:Concept
        graph.insert(&subj, &rdf::type_, &skos::Concept).unwrap();

        // skos:prefLabel
        if let Some(label) = Self::get_label(self.first("029A").unwrap()) {
            graph.insert(&subj, &skos::prefLabel, &label).unwrap();
        }

        // skos:altLabel
        for field in self.all("029@").unwrap_or_default() {
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
