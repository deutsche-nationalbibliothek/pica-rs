use pica::{Field, Record};
use regex::Regex;
use sophia::graph::MutableGraph;
use sophia::ns::{rdf, Namespace};

use std::ops::Deref;

use crate::concept::{Concept, StrLiteral};
use crate::ns::skos;

pub struct Person<'a>(pub(crate) Record<'a>);

impl<'a> Deref for Person<'a> {
    type Target = Record<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Person<'a> {
    pub fn get_label(field: &Field) -> Option<StrLiteral> {
        let mut label = String::new();

        if field.exists('a') {
            push_value!(label, field.first('a'));
            push_value!(label, field.first('d'), ", ");
            push_value!(label, field.first('c'), " ");
        } else if field.exists('P') {
            push_value!(label, field.first('P'));

            let numeration = field.first('n');
            let title = field.first('l');

            if numeration.is_some() || title.is_some() {
                label.push_str(" (");

                if numeration.is_some() {
                    if title.is_some() {
                        push_value!(label, numeration, "", ", ");
                    } else {
                        push_value!(label, numeration);
                    }
                }

                push_value!(label, title);
                label.push(')');
            }
        }

        if !label.is_empty() {
            Some(StrLiteral::new_lang(label, "de").unwrap())
        } else {
            None
        }
    }

    fn get_time_data(&self) -> Option<String> {
        let mut time_data = String::new();

        let field = self.iter().find(|field| {
            field.iter().any(|subfield| {
                subfield.code() == '4' && subfield.value() == "datl"
            })
        });

        if let Some(field) = field {
            let from = field.first('a');
            let to = field.first('b');

            if from.is_some() && to.is_some() {
                time_data.push_str(&format!(
                    " ({}-{})",
                    &from.unwrap(),
                    &to.unwrap()
                ));
            } else if let Some(time) = field.first('c') {
                time_data.push_str(&format!(" ({})", &time));
            } else if let Some(time) = field.first('d') {
                time_data.push_str(&format!(" ({})", &time));
            }
        }

        if !time_data.is_empty() {
            Some(time_data)
        } else {
            None
        }
    }
}

impl<'a> Concept for Person<'a> {
    fn skosify<G: MutableGraph>(&self, graph: &mut G) {
        let gnd = Namespace::new("http://d-nb.info/gnd/").unwrap();
        let idn = self.first("003@").unwrap().first('0').unwrap();
        let re = Regex::new(r"([^,]+),\s([^,]+)$").unwrap();
        let subj = gnd.get(&idn).unwrap();

        // skos:Concept
        graph.insert(&subj, &rdf::type_, &skos::Concept).unwrap();

        // skos:prefLabel
        if let Some(label) = Self::get_label(self.first("028A").unwrap()) {
            let label = if let Some(time_data) = self.get_time_data() {
                StrLiteral::new_lang(
                    format!("{}{}", label.txt(), time_data),
                    "de",
                )
                .unwrap()
            } else {
                label
            };

            graph.insert(&subj, &skos::prefLabel, &label).unwrap();
        }

        // skos:altLabel
        for field in self.all("028@") {
            if let Some(label) = Self::get_label(field) {
                graph.insert(&subj, &skos::altLabel, &label).unwrap();
            }
        }

        // skos:hiddenLabel
        if let Some(label) = Self::get_label(self.first("028A").unwrap()) {
            graph.insert(&subj, &skos::hiddenLabel, &label).unwrap();

            if let Some(captures) = re.captures(label.txt()) {
                let obj = StrLiteral::new_lang(
                    format!(
                        "{} {}",
                        captures.get(2).unwrap().as_str(),
                        captures.get(1).unwrap().as_str()
                    ),
                    "de",
                )
                .unwrap();

                graph.insert(&subj, &skos::hiddenLabel, &obj).unwrap();
            }
        }
    }
}
