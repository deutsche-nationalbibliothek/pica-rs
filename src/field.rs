//! This module provides a data structure and functions related to a PICA+
//! field.

use crate::error::ParsePicaError;
use crate::parser::parse_field;
use crate::subfield::Subfield;
use crate::Occurrence;

use nom::{combinator::all_consuming, Finish};
use serde::Serialize;
use std::borrow::Cow;
use std::ops::Deref;

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
    /// use pica::Field;
    ///
    /// let field = Field::new("003@", None, vec![]);
    /// assert_eq!(field.tag(), "003@");
    /// assert!(field.subfields().is_empty());
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
    ///
    /// # Example
    /// ```rust
    /// use pica::Field;
    ///
    /// let result = Field::decode("003@ \u{1f}0123456789X\u{1e}");
    /// assert!(result.is_ok());
    /// ```
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_field)(input).finish() {
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
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Occurrence, Subfield};
    ///
    /// let field = Field::new(
    ///     "012A",
    ///     Some(Occurrence::new("01")),
    ///     vec![Subfield::new('a', "1").unwrap()],
    /// );
    /// assert_eq!(field.occurrence(), Some(&Occurrence::new("01")));
    /// ```
    pub fn occurrence(&self) -> Option<&Occurrence> {
        self.occurrence.as_ref()
    }

    /// Returns the subfields of the field.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Subfield};
    ///
    /// let field =
    ///     Field::new("012A", None, vec![Subfield::new('a', "123").unwrap()]);
    /// assert_eq!(*field.subfields(), vec![Subfield::new('a', "123").unwrap()]);
    /// ```
    pub fn subfields(&self) -> &Vec<Subfield> {
        &self.subfields
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
    ///
    /// assert_eq!(field.len(), 2);
    /// ```
    fn deref(&self) -> &Vec<Subfield<'a>> {
        &self.subfields
    }
}
