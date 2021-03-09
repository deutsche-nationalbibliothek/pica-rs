use pica::{Field, Record};
use rdf::node::Node;
use rdf::uri::Uri;
use std::ops::Deref;

use crate::skos;
use skos::{Concept, Result};

pub struct Work<'a>(pub(crate) Record<'a>);

impl<'a> Deref for Work<'a> {
    type Target = Record<'a>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Work<'a> {
    fn get_label(
        &self,
        field: &Field,
        prefix: &str,
        predicate: &str,
    ) -> Result {
        let mut result = String::new();

        for subfield in field.iter() {
            let value = String::from_utf8(subfield.value().to_vec()).unwrap();
            match subfield.code() {
                'a' => {
                    result.push_str(&value.replace('@', ""));
                }
                'g' => {
                    result.push_str(&format!(" ({})", value));
                }
                'n' => {
                    result.push_str(&format!(" {}", value));
                }
                'h' | 'f' => {
                    result.push_str(&format!(", {}", value));
                }
                'p' | 's' => {
                    result.push_str(&format!(" / {}", value));
                }
                _ => continue,
            }
        }

        if !result.is_empty() {
            if !prefix.is_empty() {
                result = format!("{} : {}", prefix, result);
            }

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

    fn get_prefix(&self) -> String {
        let mut result = String::new();

        for tag in vec!["028R", "065R", "029R", "030R"] {
            if let Some(field) = self.first(tag) {
                if field.iter().any(|subfield| {
                    subfield.code() == '4'
                        && (subfield.value() == "aut1"
                            || subfield.value() == "kom1"
                            || subfield.value() == "kue1")
                }) {
                    match tag {
                        "028R" => {
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

                            break;
                        }
                        "065R" => {
                            for subfield in field.iter() {
                                let value = String::from_utf8(
                                    subfield.value().to_vec(),
                                )
                                .unwrap();

                                match subfield.code() {
                                    'a' => {
                                        result
                                            .push_str(&value.replace('@', ""));
                                    }
                                    'g' | 'z' => result
                                        .push_str(&format!(" ({})", value)),
                                    'x' => result
                                        .push_str(&format!(" / {}", value)),
                                    _ => continue,
                                }
                            }

                            break;
                        }
                        "029R" => {
                            for subfield in field.iter() {
                                let value = String::from_utf8(
                                    subfield.value().to_vec(),
                                )
                                .unwrap();
                                match subfield.code() {
                                    'a' => {
                                        result
                                            .push_str(&value.replace('@', ""));
                                    }
                                    'g' => {
                                        result
                                            .push_str(&format!(" ({})", value));
                                    }
                                    'x' | 'b' => {
                                        result
                                            .push_str(&format!(" / {}", value));
                                    }
                                    'n' => {
                                        result
                                            .push_str(&format!(", {}", value));
                                    }
                                    _ => continue,
                                }
                            }

                            break;
                        }
                        "030R" => {
                            let mut parens = String::new();
                            let check = vec!['n', 'd', 'c'];

                            for subfield in field.iter() {
                                let value = String::from_utf8(
                                    subfield.value().to_vec(),
                                )
                                .unwrap();

                                if !check.contains(&subfield.code()) {
                                    if !parens.is_empty() {
                                        result.push_str(&format!(
                                            " ({})",
                                            parens
                                        ));
                                        parens.clear();
                                    }
                                }

                                match subfield.code() {
                                    'a' => {
                                        result
                                            .push_str(&value.replace('@', ""));
                                    }
                                    'e' | 'x' | 'b' => {
                                        result
                                            .push_str(&format!(" / {}", value));
                                    }
                                    'g' => {
                                        if parens.is_empty() {
                                            result.push_str(&format!(
                                                " ({})",
                                                value
                                            ))
                                        } else {
                                            parens.push_str(&format!(
                                                " ({})",
                                                value
                                            ))
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

                            break;
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        result
    }
}

impl<'a> Concept for Work<'a> {
    fn uri(&self) -> Uri {
        Uri::new(format!(
            "http://d-nb.info/gnd/{}",
            self.first("003@").unwrap().first('0').unwrap()
        ))
    }

    fn pref_label(&self) -> Result {
        if let Some(field) = self.first("022A") {
            self.get_label(&field, &self.get_prefix(), "prefLabel")
        } else {
            None
        }
    }

    fn alt_labels(&self) -> Vec<(Node, Node)> {
        let mut result = Vec::new();

        for field in self.all("022@") {
            if let Some(label) =
                self.get_label(&field, &self.get_prefix(), "altLabel")
            {
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
