use bstr::BStr;

use crate::parse;

#[derive(Debug, PartialEq)]
pub struct Subfield<'a> {
    pub(crate) name: char,
    pub(crate) value: &'a BStr,
}

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub(crate) name: &'a BStr,
    pub(crate) occurrence: Option<&'a BStr>,
    pub(crate) subfields: Vec<Subfield<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Record<'a>(pub(crate) Vec<Field<'a>>);

impl<'a> Record<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ()> {
        parse::record(data).map(|(_, s)| s).map_err(|_| ())
    }
}
