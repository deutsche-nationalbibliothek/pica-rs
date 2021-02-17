//! This module provides a data structure and functions related to a PICA+
//! record.

use crate::error::ParsePicaError;
use crate::parser::{parse_field, parse_record, parse_subfield};
use crate::select::{Outcome, Selector};
use crate::{Occurrence, Path};

use nom::{combinator::all_consuming, Finish};

use serde::Serialize;
use std::borrow::Cow;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Subfield<'a> {
    pub(crate) name: char,
    pub(crate) value: Cow<'a, str>,
}

impl<'a> Subfield<'a> {
    /// Crates a new subfield
    ///
    /// # Arguments
    ///
    /// * `name` - An alpha-numeric ([0-9A-Za-z]) subfield code.
    /// * `value` - A string or string slice holding the subfield value.
    pub fn new<S>(name: char, value: S) -> Result<Self, ParsePicaError>
    where
        S: Into<Cow<'a, str>>,
    {
        let value = value.into();

        if !name.is_ascii_alphanumeric()
            || value.contains(&['\u{1e}', '\u{1f}'][..])
        {
            Err(ParsePicaError::InvalidSubfield)
        } else {
            Ok(Subfield { name, value })
        }
    }

    /// Decodes a subfield
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_subfield)(input).finish() {
            Ok((_, subfield)) => Ok(subfield),
            _ => Err(ParsePicaError::InvalidSubfield),
        }
    }

    /// Encodes a subfield
    pub fn encode(&self) -> String {
        format!("\u{1f}{}{}", self.name, self.value)
    }

    /// Returns the name of the subfield.
    pub fn name(&self) -> char {
        self.name
    }

    /// Returns the value of the subfield
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }

    /// Returns the subfield as an human readable string
    pub fn pretty(&self) -> String {
        format!("${} {}", self.name, self.value)
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Field<'a> {
    pub(crate) tag: Cow<'a, str>,
    pub(crate) occurrence: Option<Occurrence<'a>>,
    pub(crate) subfields: Vec<Subfield<'a>>,
}

impl<'a> Field<'a> {
    /// Create a new field.
    ///
    /// # Example
    /// ```
    pub fn new<S>(
        tag: S,
        occurrence: Option<Occurrence<'a>>,
        subfields: Vec<Subfield<'a>>,
    ) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            tag: tag.into(),
            occurrence,
            subfields,
        }
    }

    /// Decodes a field
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice holding a PICA+ encoded field
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_field)(input).finish() {
            Ok((_, field)) => Ok(field),
            _ => Err(ParsePicaError::InvalidField),
        }
    }

    /// Returns the tag of the field.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the occurrence of the field.
    pub fn occurrence(&self) -> Option<&Occurrence> {
        self.occurrence.as_ref()
    }

    /// Returns the subfields of the field.
    pub fn subfields(&self) -> &Vec<Subfield> {
        &self.subfields
    }

    /// Returns the field as an pretty formatted string.
    pub fn pretty(&self) -> String {
        let mut pretty_str = String::from(self.tag.clone());

        if let Some(occurrence) = self.occurrence() {
            pretty_str.push('/');
            pretty_str.push_str(occurrence)
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
    fn deref(&self) -> &Vec<Subfield<'a>> {
        &self.subfields
    }
}

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct Record<'a>(Vec<Field<'a>>);

impl<'a> Record<'a> {
    /// Creates a new record
    ///
    /// # Arguments
    ///
    /// * A vector of [`Field`]s
    pub fn new(fields: Vec<Field<'a>>) -> Self {
        Self(fields)
    }

    /// Decodes a record
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice holding a PICA+ record
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_record)(input).finish() {
            Ok((_remaining, record)) => Ok(record),
            _ => Err(ParsePicaError::InvalidRecord),
        }
    }

    /// Returns the record as an pretty formatted string.
    pub fn pretty(&self) -> String {
        String::from(
            &*self
                .iter()
                .map(|s| s.pretty())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    pub fn path<S>(&self, path_str: S) -> Vec<&str>
    where
        S: Into<Cow<'a, str>>,
    {
        let path_str = path_str.into();
        let path = Path::decode(&path_str).unwrap();
        let mut result: Vec<&str> = Vec::new();

        for field in &self.0 {
            if field.tag() == path.tag()
                && field.occurrence().as_deref() == path.occurrence()
            {
                for subfield in &field.subfields {
                    if subfield.name() == path.name() {
                        result.push(subfield.value());
                    }
                }
            }
        }

        if let Some(index) = path.index() {
            if let Some(value) = result.get(index) {
                return vec![value];
            } else {
                return vec![];
            }
        }

        result
    }

    pub fn select(&self, selector: &Selector) -> Outcome {
        self.iter()
            .filter(|field| selector.tag == field.tag())
            .filter(|field| selector.occurrence == field.occurrence())
            .map(|field| field.subfields())
            .map(|subfields| {
                selector
                    .subfields
                    .iter()
                    .map(|name| {
                        subfields
                            .iter()
                            .filter(|subfield| subfield.name() == *name)
                            .map(|subfield| vec![subfield.value()])
                            .collect::<Vec<Vec<&str>>>()
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
            .fold(Outcome::default(), |acc, x| acc + x)
    }
}

impl<'a> Deref for Record<'a> {
    type Target = Vec<Field<'a>>;

    fn deref(&self) -> &Vec<Field<'a>> {
        &self.0
    }
}
