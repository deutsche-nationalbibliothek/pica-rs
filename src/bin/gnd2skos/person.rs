use pica::{Field, Record};
use rdf::node::Node;
use rdf::uri::Uri;
use std::ops::Deref;

use crate::concept::{Concept, Result};

pub struct Person<'a>(pub(crate) Record<'a>);

impl<'a> Deref for Person<'a> {
    type Target = Record<'a>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Person<'a> {
    pub fn get_name(field: &Field) -> String {
        let mut result = String::new();

        if let Some(surname) = field.first('a') {
            result.push_str(&surname);

            if let Some(firstname) = field.first('d') {
                result.push_str(", ");
                result.push_str(&firstname);
            }

            if let Some(prefix) = field.first('c') {
                result.push_str(" ");
                result.push_str(&prefix);
            }
        } else if let Some(name) = field.first('P') {
            result.push_str(&name);

            let numeration = field.first('n');
            let title = field.first('l');

            if numeration.is_some() || title.is_some() {
                result.push_str(" (");

                if let Some(numeration) = numeration {
                    result.push_str(&numeration);
                    if title.is_some() {
                        result.push_str(", ");
                    }
                }

                if let Some(title) = title {
                    result.push_str(&title);
                }

                result.push(')');
            }
        }

        result
    }

    fn get_label(
        &self,
        field: &Field,
        time_data: bool,
        predicate: &str,
    ) -> Result {
        let mut result = Self::get_name(field);
        if time_data && !result.is_empty() {
            let field = self
                .iter()
                .filter(|field| {
                    field.iter().any(|subfield| {
                        subfield.code() == '4' && subfield.value() == "datl"
                    })
                })
                .nth(0);

            if let Some(field) = field {
                let from = field.first('a');
                let to = field.first('b');

                if from.is_some() && to.is_some() {
                    result.push_str(&format!(
                        " ({}-{})",
                        &from.unwrap(),
                        &to.unwrap()
                    ));
                } else if let Some(time) = field.first('c') {
                    result.push_str(&format!(" ({})", &time));
                } else if let Some(time) = field.first('d') {
                    result.push_str(&format!(" ({})", &time));
                }
            }
        }

        // result = result.replace('"', "\\\"");
        // result = result.replace("'", "\\\'");

        if !result.is_empty() {
            return Some((
                Node::UriNode {
                    uri: Uri::new(skos!(predicate)),
                },
                Node::LiteralNode {
                    literal: result,
                    data_type: None,
                    language: None,
                },
            ));
        }

        None
    }
}

impl<'a> Concept for Person<'a> {
    fn uri(&self) -> Uri {
        Uri::new(format!(
            "http://d-nb.info/gnd/{}",
            self.first("003@").unwrap().first('0').unwrap()
        ))
    }

    fn pref_label(&self) -> Result {
        if let Some(field) = self.first("028A") {
            self.get_label(&field, true, "prefLabel")
        } else {
            None
        }
    }

    fn alt_labels(&self) -> Vec<(Node, Node)> {
        let mut result = Vec::new();

        for field in self.all("028@") {
            if let Some(label) = self.get_label(&field, false, "altLabel") {
                result.push(label)
            }
        }

        result
    }

    fn hidden_labels(&self) -> Vec<(Node, Node)> {
        let mut result = Vec::new();

        for field in self.all("028A") {
            if let Some(label) = self.get_label(&field, false, "hiddenLabel") {
                result.push(label)
            }
        }

        result
    }

    fn created(&self) -> Result {
        self.date("created", &self.0, "001A")
    }

    fn modified(&self) -> Result {
        self.date("modified", &self.0, "001B")
    }
}
