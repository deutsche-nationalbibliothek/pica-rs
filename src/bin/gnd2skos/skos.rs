use chrono::NaiveDate;
use pica::Record;
use rdf::node::Node;
use rdf::uri::Uri;

pub type Result = Option<(Node, Node)>;
use crate::{dcterms, xsd};

pub trait Concept {
    fn uri(&self) -> Uri;
    fn created(&self) -> Result;
    fn modified(&self) -> Result;
    fn pref_label(&self) -> Result;
    fn alt_labels(&self) -> Vec<(Node, Node)>;

    fn date(&self, predicate: &str, record: &Record, field: &str) -> Result {
        if let Some(field) = record.first(field) {
            if let Some(subfield) = field.first('0') {
                if subfield.len() >= 13 {
                    let date_parsed =
                        NaiveDate::parse_from_str(&subfield[5..], "%d-%m-%y");
                    if let Ok(date) = date_parsed {
                        return Some((
                            Node::UriNode {
                                uri: Uri::new(dcterms!(predicate)),
                            },
                            Node::LiteralNode {
                                literal: date.format("%Y-%m-%d").to_string(),
                                data_type: Some(Uri::new(xsd!("date"))),
                                language: None,
                            },
                        ));
                    }
                }
            }
        }

        None
    }
}

pub use crate::person::Person;
