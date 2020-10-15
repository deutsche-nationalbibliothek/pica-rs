use crate::Subfield;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub struct Field<'a, 'b, 'c> {
    pub tag: Cow<'a, str>,
    pub occurrence: Cow<'b, str>,
    pub subfields: Vec<Subfield<'c>>,
}

impl<'a, 'b, 'c> Field<'a, 'b, 'c> {
    pub fn new<S, T>(
        tag: S,
        occurrence: T,
        subfields: Vec<Subfield<'c>>,
    ) -> Self
    where
        S: Into<Cow<'a, str>>,
        T: Into<Cow<'b, str>>,
    {
        Self {
            tag: tag.into(),
            occurrence: occurrence.into(),
            subfields,
        }
    }
}
