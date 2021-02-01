//! This module provides a data structure and functions related to a PICA+
//! record.

use crate::error::ParsePicaError;
use crate::parser::parse_record;
use crate::select::{Outcome, Selector};
use crate::{Field, Path};

use nom::{combinator::all_consuming, Finish};

use serde::Serialize;
use std::ops::Deref;

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct Record<'a>(Vec<Field<'a>>);

impl<'a> Record<'a> {
    /// Creates a new record
    ///
    /// # Arguments
    ///
    /// * A vector of [`Field`]s
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Record, Subfield};
    ///
    /// let record = Record::new(vec![Field::new("003@", None, vec![])]);
    /// assert_eq!(record.len(), 1);
    /// ```
    pub fn new(fields: Vec<Field<'a>>) -> Self {
        Self(fields)
    }

    /// Decodes a record
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice holding a PICA+ record
    ///
    /// # Example
    /// ```
    /// use pica::Record;
    ///
    /// let result = Record::decode("003@ \u{1f}0123456789X\u{1e}");
    /// assert!(result.is_ok());
    /// ```
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_record)(input).finish() {
            Ok((_remaining, record)) => Ok(record),
            _ => Err(ParsePicaError::InvalidRecord),
        }
    }

    /// Returns the record as an pretty formatted string.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Record, Subfield};
    ///
    /// let record = Record::new(vec![
    ///     Field::new(
    ///         "003@",
    ///         None,
    ///         vec![Subfield::new('0', "123456789X").unwrap()],
    ///     ),
    ///     Field::new(
    ///         "012A",
    ///         None,
    ///         vec![
    ///             Subfield::new('a', "123").unwrap(),
    ///             Subfield::new('b', "456").unwrap(),
    ///         ],
    ///     ),
    /// ]);
    /// assert_eq!(record.pretty(), "003@ $0 123456789X\n012A $a 123 $b 456");
    /// ```
    pub fn pretty(&self) -> String {
        String::from(
            &*self
                .iter()
                .map(|s| s.pretty())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    pub fn path(&self, path: &Path) -> Vec<&str> {
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
