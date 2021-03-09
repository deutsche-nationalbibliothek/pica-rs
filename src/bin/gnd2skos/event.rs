use pica::{Field, Record};
use rdf::node::Node;
use rdf::uri::Uri;
use std::ops::Deref;

use crate::skos;
use skos::{Concept, Result};

pub struct Event<'a>(pub(crate) Record<'a>);

impl<'a> Deref for Event<'a> {
    type Target = Record<'a>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Event<'a> {
    fn get_label(&self, field: &Field, predicate: &str) -> Result {
        let mut result = String::new();
        let mut parens = String::new();
        let check = vec!['n', 'd', 'c'];

        for subfield in field.iter() {
            let value = String::from_utf8(subfield.value().to_vec()).unwrap();

            if !check.contains(&subfield.code()) {
                if !parens.is_empty() {
                    result.push_str(&format!(" ({})", parens));
                    parens.clear();
                }
            }

            match subfield.code() {
                'a' => {
                    result.push_str(&value.replace('@', ""));
                }
                'x' | 'b' => {
                    result.push_str(&format!(" / {}", value));
                }
                'g' => {
                    if parens.is_empty() {
                        result.push_str(&format!(" ({})", value))
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
            result.push_str(&format!(" ({})", parens));
        }

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

impl<'a> Concept for Event<'a> {
    fn uri(&self) -> Uri {
        Uri::new(format!(
            "http://d-nb.info/gnd/{}",
            self.first("003@").unwrap().first('0').unwrap()
        ))
    }

    fn pref_label(&self) -> Result {
        if let Some(field) = self.first("030A") {
            self.get_label(&field, "prefLabel")
        } else {
            None
        }
    }

    fn alt_labels(&self) -> Vec<(Node, Node)> {
        let mut result = Vec::new();

        for field in self.all("030@") {
            if let Some(label) = self.get_label(&field, "altLabel") {
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
