//! Pica+ Field

use crate::error::ParsePicaError;
use crate::subfield::{parse_subfield, Subfield};
use nom::character::complete::{char, one_of};
use nom::combinator::{map, opt, recognize};
use nom::multi::{count, many0, many_m_n};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::{Finish, IResult};

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
    /// assert_eq!(field.occurrence(), None);
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
    ///
    /// # Example
    /// ```
    /// use pica::Field;
    ///
    /// let field = Field::new("012A", Some("00"), vec![]);
    /// assert_eq!(field.occurrence(), Some("00"));
    /// ```
    pub fn occurrence(&self) -> Option<&str> {
        self.occurrence.as_deref()
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

pub(crate) fn parse_field_tag(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        one_of("012"),
        count(one_of("0123456789"), 2),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
    )))(i)
}

pub(crate) fn parse_field_occurrence(i: &str) -> IResult<&str, &str> {
    preceded(char('/'), recognize(many_m_n(2, 3, one_of("0123456789"))))(i)
}

pub(crate) fn parse_field(i: &str) -> IResult<&str, Field> {
    terminated(
        map(
            pair(
                pair(parse_field_tag, opt(parse_field_occurrence)),
                preceded(char(' '), many0(parse_subfield)),
            ),
            |((tag, occurrence), subfields)| {
                Field::new(tag, occurrence, subfields)
            },
        ),
        char('\u{1e}'),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_tag() {
        for tag in vec!["000A", "100A", "200A", "000A", "000@"] {
            assert_eq!(parse_field_tag(tag), Ok(("", tag)));
        }

        for tag in vec!["300A", "0A0A", "00AA", "0001"] {
            assert!(parse_field_tag(tag).is_err())
        }
    }

    #[test]
    fn test_parse_field_occurrence() {
        assert_eq!(parse_field_occurrence("/00"), Ok(("", "00")));
        assert_eq!(parse_field_occurrence("/001"), Ok(("", "001")));
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field("012A/00 \u{1e}"),
            Ok(("", Field::new("012A", Some("00"), vec![])))
        );
        assert_eq!(
            parse_field("012A \u{1e}"),
            Ok(("", Field::new("012A", None, vec![])))
        );
        assert_eq!(
            parse_field("003@ \u{1f}0123456789\u{1e}"),
            Ok((
                "",
                Field::new(
                    "003@",
                    None,
                    vec![Subfield::new('0', "123456789").unwrap()]
                )
            ))
        );
    }
}
