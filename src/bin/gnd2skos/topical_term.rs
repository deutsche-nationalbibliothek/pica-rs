use pica::{Field, Record};
use rdf::node::Node;
use rdf::uri::Uri;
use std::ops::Deref;

use crate::concept::{Concept, Result};

pub struct TopicalTerm<'a>(pub(crate) Record<'a>);

impl<'a> Deref for TopicalTerm<'a> {
    type Target = Record<'a>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TopicalTerm<'a> {
    fn get_label(&self, field: &Field, predicate: &str) -> Result {
        let mut result = String::new();

        if let Some(term) = field.first('a') {
            result.push_str(&term);
            result = result.replace('@', "");

            let addition = field.all('g');
            if !addition.is_empty() {
                result.push_str(&format!(" ({})", addition.join(", ")));
            }

            let subdivision = field.all('x');
            if !subdivision.is_empty() {
                result.push_str(&format!(" / {}", subdivision.join(" / ")));
            }
        }

        result = result.replace('"', "\\\"");
        result = result.replace("'", "\\\'");

        if !result.is_empty() {
            return Some((
                Node::UriNode {
                    uri: Uri::new(skos!(predicate)),
                },
                Node::LiteralNode {
                    literal: result,
                    data_type: None,
                    language: Some("de".to_string()),
                },
            ));
        }

        None
    }
}

impl<'a> Concept for TopicalTerm<'a> {
    fn uri(&self) -> Uri {
        Uri::new(format!(
            "http://d-nb.info/gnd/{}",
            self.first("003@").unwrap().first('0').unwrap()
        ))
    }

    fn pref_label(&self) -> Result {
        if let Some(field) = self.first("041A") {
            self.get_label(&field, "prefLabel")
        } else {
            None
        }
    }

    fn alt_labels(&self) -> Vec<(Node, Node)> {
        let mut result = Vec::new();

        for field in self.all("041@") {
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
