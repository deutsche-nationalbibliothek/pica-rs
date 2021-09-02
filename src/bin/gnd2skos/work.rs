use bstr::ByteSlice;
use pica::{Field, StringRecord};
use sophia::graph::MutableGraph;
use sophia::ns::{rdf, Namespace};
use std::ops::Deref;

use crate::concept::{Concept, StrLiteral};
use crate::corporate_body::CorporateBody;
use crate::event::Event;
use crate::geoplace::GeoPlace;
use crate::ns::skos;
use crate::person::Person;
use crate::AppContext;

pub struct Work(pub(crate) StringRecord);

impl Deref for Work {
    type Target = StringRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Work {
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
                'n' => {
                    label.push_str(&format!(" {}", value));
                }
                'h' | 'f' => {
                    label.push_str(&format!(", {}", value));
                }
                'p' | 's' | 'x' => {
                    label.push_str(&format!(" / {}", value));
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

    pub fn get_prefix(&self) -> Option<StrLiteral> {
        for tag in &["028R", "065R", "029R", "030R"] {
            for field in self.all(tag).unwrap_or_default() {
                let relation_exists = field.iter().any(|subfield| {
                    subfield.code() == '4'
                        && (subfield.value() == "aut1"
                            || subfield.value() == "kom1"
                            || subfield.value() == "kue1")
                });

                if relation_exists {
                    let prefix = match *tag {
                        "028R" => Person::get_label(field),
                        "029R" => CorporateBody::get_label(field),
                        "030R" => Event::get_label(field),
                        "065R" => GeoPlace::get_label(field),
                        _ => unreachable!(),
                    };

                    if prefix.is_some() {
                        return prefix;
                    }
                }
            }
        }

        None
    }
}

impl Concept for Work {
    fn skosify<G: MutableGraph>(&self, graph: &mut G, ctx: &AppContext) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();
        let idn = self.first("003@").unwrap().first('0').unwrap();
        let subj = gnd.get(idn.to_str().unwrap()).unwrap();
        let prefix = self.get_prefix();

        // skos:Concept
        graph.insert(&subj, &rdf::type_, &skos::Concept).unwrap();

        // skos:prefLabel
        if let Some(label) = Self::get_label(self.first("022A").unwrap()) {
            if !ctx
                .label_ignore_list
                .contains(label.txt().to_string(), idn.to_string())
            {
                if let Some(ref prefix) = prefix {
                    let label = StrLiteral::new_lang(
                        format!("{} : {}", prefix.txt(), label.txt()),
                        "de",
                    )
                    .unwrap();

                    graph.insert(&subj, &skos::prefLabel, &label).unwrap();
                } else {
                    graph.insert(&subj, &skos::prefLabel, &label).unwrap();
                }
            }
        }

        // skos:altLabel
        for field in self.all("022@").unwrap_or_default() {
            if let Some(label) = Self::get_label(field) {
                if !ctx
                    .label_ignore_list
                    .contains(label.txt().to_string(), idn.to_string())
                {
                    if let Some(ref prefix) = prefix {
                        let label = StrLiteral::new_lang(
                            format!("{} : {}", prefix.txt(), label.txt()),
                            "de",
                        )
                        .unwrap();

                        graph.insert(&subj, &skos::altLabel, &label).unwrap();
                    } else {
                        graph.insert(&subj, &skos::altLabel, &label).unwrap();
                    }
                }
            }
        }

        // skos:broader or skos:related
        for field in ["022R", "028R", "029R", "030R", "041R", "065R"] {
            self.add_relations(&subj, self.all(field), graph, ctx.args);
        }
    }
}
