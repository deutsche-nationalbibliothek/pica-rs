//! This module provides a data structure and functions related to a PICA+
//! field.

use crate::error::ParsePicaError;
use crate::filter::{BooleanOp, ComparisonOp, SubfieldFilter};
use crate::parser::parse_field;
use crate::subfield::Subfield;

use nom::Finish;
use regex::Regex;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Field<'a> {
    pub(crate) tag: Cow<'a, str>,
    pub(crate) occurrence: Option<Cow<'a, str>>,
    pub(crate) subfields: Vec<Subfield<'a>>,
}

impl<'a> Field<'a> {
    /// Create a new field.
    ///
    /// # Example
    /// ```
    /// use pica::Field;
    ///
    /// let field = Field::new("003@", None, vec![]);
    /// assert_eq!(field.tag(), "003@");
    /// assert_eq!(field.subfields(), vec![]);
    /// ```
    pub fn new<S>(
        tag: S,
        occurrence: Option<S>,
        subfields: Vec<Subfield<'a>>,
    ) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            tag: tag.into(),
            occurrence: occurrence.map(|o| o.into()),
            subfields,
        }
    }

    pub fn decode(s: &'a str) -> Result<Self, ParsePicaError> {
        match parse_field(s).finish() {
            Ok((_, field)) => Ok(field),
            _ => Err(ParsePicaError::InvalidField),
        }
    }

    /// Returns the tag of the field.
    ///
    /// # Example
    /// ```
    /// use pica::Field;
    ///
    /// let field = Field::new("003@", None, vec![]);
    /// assert_eq!(field.tag(), "003@");
    /// ```
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the occurrence of the field.
    pub fn occurrence(&self) -> &Option<Cow<'a, str>> {
        &self.occurrence
    }

    /// Returns the subfields of the field.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Subfield};
    ///
    /// let field =
    ///     Field::new("012A", None, vec![Subfield::new('a', "123").unwrap()]);
    /// assert_eq!(field.subfields(), vec![Subfield::new('a', "123").unwrap()]);
    /// ```
    pub fn subfields(&self) -> &[Subfield] {
        &self.subfields
    }

    pub fn matches(&self, filter: &SubfieldFilter) -> bool {
        match filter {
            SubfieldFilter::Comparison(name, op, values) => match op {
                ComparisonOp::Eq => self.subfields.iter().any(|subfield| {
                    subfield.name() == *name && subfield.value() == values[0]
                }),
                ComparisonOp::Ne => self.subfields.iter().all(|subfield| {
                    subfield.name() == *name && subfield.value() != values[0]
                }),
                ComparisonOp::StartsWith => {
                    self.subfields.iter().any(|subfield| {
                        subfield.name() == *name
                            && subfield.value().starts_with(&values[0])
                    })
                }
                ComparisonOp::EndsWith => {
                    self.subfields.iter().any(|subfield| {
                        subfield.name() == *name
                            && subfield.value().ends_with(&values[0])
                    })
                }
                ComparisonOp::Re => {
                    let re = Regex::new(&values[0]).unwrap();
                    self.subfields.iter().any(|subfield| {
                        subfield.name() == *name
                            && re.is_match(subfield.value())
                    })
                }
                ComparisonOp::In => self.subfields.iter().any(|subfield| {
                    subfield.name() == *name
                        && values.contains(&String::from(subfield.value()))
                }),
            },
            SubfieldFilter::Boolean(lhs, op, rhs) => match op {
                BooleanOp::And => self.matches(lhs) && self.matches(rhs),
                BooleanOp::Or => self.matches(lhs) || self.matches(rhs),
            },
            SubfieldFilter::Grouped(filter) => self.matches(filter),
            SubfieldFilter::Not(filter) => !self.matches(filter),
            SubfieldFilter::Exists(name) => self
                .subfields
                .iter()
                .any(|subfield| subfield.name() == *name),
        }
    }

    /// Returns the field as an pretty formatted string.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Subfield};
    ///
    /// let field = Field::new(
    ///     "012A",
    ///     None,
    ///     vec![
    ///         Subfield::new('a', "123").unwrap(),
    ///         Subfield::new('b', "456").unwrap(),
    ///     ],
    /// );
    /// assert_eq!(field.pretty(), "012A $a 123 $b 456");
    /// ```
    pub fn pretty(&self) -> String {
        let mut pretty_str = String::from(self.tag.clone());

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
