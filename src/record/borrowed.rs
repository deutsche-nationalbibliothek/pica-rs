//! This module provides all data types related to a PICA+ record.

use crate::record::parse_record;
use bstr::BStr;

#[derive(Debug, PartialEq)]
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
}

#[derive(Debug, PartialEq)]
pub struct Occurrence<'a>(pub(crate) &'a BStr);

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub(crate) tag: &'a BStr,
    pub(crate) occurrence: Option<Occurrence<'a>>,
    pub(crate) subfields: Vec<Subfield<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Record<'a>(pub(crate) Vec<Field<'a>>);

impl<'a> Record<'a> {
    /// Parses a record from a byte slice.
    #[allow(clippy::result_unit_err)]
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ()> {
        parse_record(data).map(|(_, record)| record).map_err(|_| ())
    }
}
