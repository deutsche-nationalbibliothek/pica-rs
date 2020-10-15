use crate::Subfield;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub struct Field<'a> {
    pub tag: Cow<'a, str>,
    pub occurrence: Cow<'a, str>,
    pub subfields: Vec<Subfield<'a>>,
}

impl<'a> Field<'a> {
    pub fn new<S>(tag: S, occurrence: S, subfields: Vec<Subfield<'a>>) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            tag: tag.into(),
            occurrence: occurrence.into(),
            subfields,
        }
    }
}
