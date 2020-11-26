//! Pica+ Field

use crate::filter::{BooleanOp, ComparisonOp, SubfieldFilter};
use crate::subfield::{parse_subfield, Subfield};

use nom::character::complete::{char, one_of};
use nom::combinator::{map, opt, recognize};
use nom::multi::{count, many0, many_m_n};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::IResult;

use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Field<'a> {
    tag: Cow<'a, str>,
    occurrence: Option<Cow<'a, str>>,
    subfields: Vec<Subfield<'a>>,
}

impl<'a> Field<'a> {
    /// Create a new field.
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

    /// Returns the tag of the field.
    pub fn tag(&self) -> &str {
        self.tag.as_ref()
    }

    /// Returns the occurrence of the field.
    pub fn occurrence(&self) -> Option<&str> {
        self.occurrence.as_deref()
    }

    /// Returns the subfields of the field.
    pub fn subfields(&self) -> &[Subfield] {
        &self.subfields
    }

    pub fn matches(&self, filter: &SubfieldFilter) -> bool {
        match filter {
            SubfieldFilter::ComparisonExpr(code, op, value) => match op {
                ComparisonOp::Eq => self.subfields.iter().any(|subfield| {
                    subfield.code() == *code && subfield.value() == value
                }),
                ComparisonOp::Ne => self.subfields.iter().all(|subfield| {
                    subfield.code() == *code && subfield.value() != value
                }),
            },
            SubfieldFilter::BooleanExpr(lhs, op, rhs) => match op {
                BooleanOp::And => self.matches(lhs) && self.matches(rhs),
                BooleanOp::Or => self.matches(lhs) || self.matches(rhs),
            },
            SubfieldFilter::GroupedExpr(filter) => self.matches(filter),
            SubfieldFilter::ExistsExpr(code) => self
                .subfields
                .iter()
                .any(|subfield| subfield.code() == *code),
        }
    }

    /// Returns the field as an PICA3 formatted string.
    pub fn pica3(&self) -> String {
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
                    .map(|s| s.pica3())
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

pub fn parse_field(i: &str) -> IResult<&str, Field> {
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
    use crate::filter::{BooleanOp, ComparisonOp, SubfieldFilter};

    #[test]
    fn test_field() {
        let subfields = vec![Subfield::new('a', "123")];
        let field = Field::new("003@", Some("00"), subfields.clone());
        assert_eq!(field.tag(), "003@");
        assert_eq!(field.occurrence(), Some("00"));
        assert_eq!(field.subfields(), subfields);
        assert_eq!(field.pica3(), "003@/00 $a 123");
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field("012A/00 \u{1f}a1234567890\u{1e}"),
            Ok((
                "",
                Field::new(
                    "012A",
                    Some("00"),
                    vec![Subfield::new('a', "1234567890")]
                )
            ))
        );
    }

    #[test]
    fn test_field_matches() {
        let field =
            Field::new("012A", Some("00"), vec![Subfield::new('a', "123")]);

        let filter = SubfieldFilter::ComparisonExpr(
            'a',
            ComparisonOp::Eq,
            "123".to_string(),
        );
        assert!(field.matches(&filter));

        let filter = SubfieldFilter::BooleanExpr(
            Box::new(SubfieldFilter::ComparisonExpr(
                'a',
                ComparisonOp::Eq,
                "456".to_string(),
            )),
            BooleanOp::Or,
            Box::new(SubfieldFilter::ComparisonExpr(
                'a',
                ComparisonOp::Eq,
                "123".to_string(),
            )),
        );
        assert!(field.matches(&filter));

        let filter = SubfieldFilter::GroupedExpr(Box::new(
            SubfieldFilter::ComparisonExpr(
                'a',
                ComparisonOp::Eq,
                "123".to_string(),
            ),
        ));
        assert!(field.matches(&filter));

        let filter = SubfieldFilter::ExistsExpr('a');
        assert!(field.matches(&filter));
    }
}
