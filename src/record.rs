use crate::parser::parse_record;
use crate::select::{Outcome, Selector};
use crate::Path;

use bstr::BString;
use serde::Serialize;
use std::ops::Deref;

#[derive(Debug, PartialEq, Serialize)]
pub struct Subfield {
    #[serde(rename(serialize = "name"))]
    pub(crate) code: char,
    pub(crate) value: BString,
}

impl Subfield {
    /// Returns the subfield code.
    pub fn code(&self) -> char {
        self.code
    }

    /// Returns the subfield value.
    pub fn value(&self) -> &BString {
        &self.value
    }

    /// Returns the subfield as an human readable string.
    pub fn pretty(&self) -> String {
        let mut pretty_str = String::new();

        pretty_str.push('$');
        pretty_str.push(self.code);
        pretty_str.push(' ');
        pretty_str.push_str(&String::from_utf8(self.value.to_vec()).unwrap());
        pretty_str
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Occurrence(pub(crate) Option<BString>);

impl Deref for Occurrence {
    type Target = Option<BString>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Field {
    #[serde(rename(serialize = "name"))]
    pub(crate) tag: BString,
    pub(crate) occurrence: Occurrence,
    pub(crate) subfields: Vec<Subfield>,
}

impl Field {
    /// Returns the field as an human readable string.
    pub fn pretty(&self) -> String {
        let mut pretty_str = String::from_utf8(self.tag.to_vec()).unwrap();

        if let Some(occurrence) = &*self.occurrence {
            pretty_str.push('/');
            pretty_str
                .push_str(&String::from_utf8(occurrence.to_vec()).unwrap())
        }

        if !self.is_empty() {
            pretty_str.push(' ');
            pretty_str.push_str(
                &self
                    .iter()
                    .map(|s| s.pretty())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }

        pretty_str
    }
}

impl Deref for Field {
    type Target = Vec<Subfield>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.subfields
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Record {
    pub(crate) fields: Vec<Field>,
}

impl Record {
    /// Parses a record from a byte slice.
    #[allow(clippy::result_unit_err)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, ()> {
        parse_record(data).map(|(_, record)| record).map_err(|_| ())
    }

    /// Returns all subfield values of a given path.
    pub fn path(&self, path: &Path) -> Vec<BString> {
        let mut result: Vec<BString> = Vec::new();

        for field in &self.fields {
            if field.tag == path.tag && field.occurrence == path.occurrence {
                for subfield in &field.subfields {
                    if subfield.code == path.code {
                        result.push(subfield.value.to_owned())
                    }
                }
            }
        }

        result
    }

    /// Returns the record as an human readable string.
    pub fn pretty(&self) -> String {
        self.fields
            .iter()
            .map(|s| s.pretty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn select(&self, selector: &Selector) -> Outcome {
        let result = self
            .iter()
            .filter(|field| selector.tag == field.tag)
            .filter(|field| selector.occurrence == field.occurrence)
            .filter(|field| {
                if let Some(filter) = &selector.filter {
                    filter.matches(&field)
                } else {
                    true
                }
            })
            .map(|field| &field.subfields)
            .map(|subfields| {
                selector
                    .subfields
                    .iter()
                    .map(|code| {
                        subfields
                            .iter()
                            .filter(|subfield| subfield.code == *code)
                            .map(|subfield| vec![subfield.value().to_owned()])
                            .collect::<Vec<Vec<BString>>>()
                    })
                    .map(|x| {
                        if x.is_empty() {
                            Outcome::one()
                        } else {
                            Outcome(x)
                        }
                    })
                    .fold(Outcome::default(), |acc, x| acc * x)
            })
            .fold(Outcome::default(), |acc, x| acc + x);

        if result.is_empty() {
            let mut values: Vec<BString> =
                Vec::with_capacity(selector.subfields.len());
            for _ in 0..selector.subfields.len() {
                values.push(BString::from(""));
            }

            Outcome::from_values(values)
        } else {
            result
        }
    }
}

impl Deref for Record {
    type Target = Vec<Field>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}
