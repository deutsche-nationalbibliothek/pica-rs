//! This module provides all data types related to a PICA+ record.

use crate::record::{owned, parse_record};
use crate::Path;

use bstr::{BStr, BString};
use serde::Serialize;
use std::ops::Deref;

#[derive(Debug, PartialEq, Serialize)]
pub struct Subfield<'a> {
    pub(crate) code: char,
    pub(crate) value: &'a BStr,
}

impl<'a> Subfield<'a> {
    /// Returns the subfield code.
    pub fn code(&self) -> char {
        self.code
    }

    /// Returns the subfield value.
    pub fn value(&self) -> &'a BStr {
        self.value
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

#[derive(Debug, PartialEq, Clone, Serialize, Copy)]
pub struct Occurrence<'a>(pub(crate) &'a BStr);

impl<'a> Deref for Occurrence<'a> {
    type Target = BStr;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Field<'a> {
    pub(crate) tag: &'a BStr,
    pub(crate) occurrence: Option<Occurrence<'a>>,
    pub(crate) subfields: Vec<Subfield<'a>>,
}

impl<'a> Field<'a> {
    /// Returns the field as an human readable string.
    pub fn pretty(&self) -> String {
        let mut pretty_str = String::from_utf8(self.tag.to_vec()).unwrap();

        if let Some(ref occurrence) = self.occurrence {
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

impl<'a> Deref for Field<'a> {
    type Target = Vec<Subfield<'a>>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.subfields
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Record<'a> {
    pub(crate) fields: Vec<Field<'a>>,
}

impl<'a> Record<'a> {
    /// Parses a record from a byte slice.
    #[allow(clippy::result_unit_err)]
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ()> {
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

    pub fn to_owned(self) -> owned::Record {
        owned::Record::from(self)
    }
}

impl<'a> Deref for Record<'a> {
    type Target = Vec<Field<'a>>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}
