use pica::{Field, Record};
use rdf::node::Node;
use rdf::uri::Uri;
use std::ops::Deref;

use crate::skos;
use skos::{Concept, Result};

pub struct GeoPlace<'a>(pub(crate) Record<'a>);

impl<'a> Deref for GeoPlace<'a> {
    type Target = Record<'a>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> GeoPlace<'a> {
    fn get_label(&self, field: &Field, predicate: &str) -> Result {
        let mut result = String::new();

        for subfield in field.iter() {
            let value = String::from_utf8(subfield.value().to_vec()).unwrap();

            match subfield.code() {
                'a' => {
                    result.push_str(&value.replace('@', ""));
                }
                'g' | 'z' => result.push_str(&format!(" ({})", value)),
                'x' => result.push_str(&format!(" / {}", value)),
                _ => continue,
            }
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

impl<'a> Concept for GeoPlace<'a> {
    fn uri(&self) -> Uri {
        Uri::new(format!(
            "http://d-nb.info/gnd/{}",
            self.first("003@").unwrap().first('0').unwrap()
        ))
    }

    fn pref_label(&self) -> Result {
        if let Some(field) = self.first("065A") {
            self.get_label(&field, "prefLabel")
        } else {
            None
        }
    }

    fn alt_labels(&self) -> Vec<(Node, Node)> {
        let mut result = Vec::new();

        for field in self.all("065@") {
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
