//! Pica+ Field

use crate::error::ParsePicaError;
use crate::parser::parse_field;
use crate::Subfield;
use nom::Finish;
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Field<'a> {
    pub(crate) tag: String,
    pub(crate) occurrence: Option<String>,
    pub(crate) subfields: Vec<Subfield<'a>>,
}

impl Field {
    /// Create a new field.
    pub fn new<S>(
        tag: S,
        occurrence: Option<S>,
        subfields: Vec<Subfield>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            tag: tag.into(),
            occurrence: occurrence.map(|o| o.into()),
            subfields,
        }
    }

    /// Returns the tag of the field.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the occurrence of the field.
    pub fn occurrence(&self) -> Option<&str> {
        self.occurrence.as_deref()
    }

    /// Returns the subfields of the field.
    pub fn subfields(&self) -> &[Subfield] {
        &self.subfields
    }

    /// Returns the field as an pretty formatted string.
    pub fn pretty(&self) -> String {
        let mut pretty_str = String::from(&self.tag);

        if let Some(occurrence) = self.occurrence() {
            pretty_str.push('/');
            pretty_str.push_str(&occurrence);
        }

        if !self.subfields.is_empty() {
            pretty_str.push(' ');
            pretty_str.push_str(
                &self
                    .subfields
                    .iter()
                    .map(|s| s.pretty())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }

        pretty_str
    }
}

impl FromStr for Field {
    type Err = ParsePicaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_field(s).finish() {
            Ok((_, field)) => Ok(field),
            _ => Err(ParsePicaError::InvalidField),
        }
    }
}
