use bstr::ByteSlice;
use pica::{Field, StringRecord};
use sophia::graph::MutableGraph;
use sophia::ns::{rdf, Namespace};
use std::ops::Deref;

use crate::concept::{Concept, StrLiteral};
use crate::ns::skos;
use crate::AppContext;

pub(crate) struct TopicalTerm(pub(crate) StringRecord);

impl Deref for TopicalTerm {
    type Target = StringRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TopicalTerm {
    pub(crate) fn get_label(field: &Field) -> Option<StrLiteral> {
        let mut label = String::new();

        if field.contains_code('a') {
            push_value!(label, field.first('a'));
            push_list!(label, field.all('g').unwrap_or_default(), ", ", " (", ")");
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
    fn skosify<G: MutableGraph>(&self, graph: &mut G, ctx: &AppContext) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();
        let idn = self.first("003@").unwrap().first('0').unwrap();
        let subj = gnd.get(idn.to_str().unwrap()).unwrap();

        // skos:Concept
        graph.insert(&subj, &rdf::type_, &skos::Concept).unwrap();

        // skos:prefLabel
        for field in self.all("041A").unwrap_or_default() {
            if let Some(label) = Self::get_label(field) {
                if !ctx
                    .label_ignore_list
                    .contains(label.txt().to_string(), idn.to_string())
                {
                    graph.insert(&subj, &skos::prefLabel, &label).unwrap();
                }
            }
        }

        // skos:altLabel
        for field in self.all("041@").unwrap_or_default() {
            if let Some(label) = Self::get_label(field) {
                if !ctx
                    .label_ignore_list
                    .contains(label.txt().to_string(), idn.to_string())
                {
                    graph.insert(&subj, &skos::altLabel, &label).unwrap();
                }
            }
        }

        // skos:broader or skos:related
        for field in ["022R", "028R", "029R", "030R", "041R", "065R"] {
            self.add_relations(&subj, self.all(field), graph, ctx.args);
        }
    }
}
