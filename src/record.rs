//! Pica+ Record

use crate::field::{parse_field, Field};
use crate::filter::{BooleanOp, Filter};
use nom::combinator::map;
use nom::multi::many1;
use nom::{Finish, IResult};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsePicaError;

#[derive(Debug, PartialEq)]
pub struct Record<'a>(Vec<Field<'a>>);

impl<'a> Record<'a> {
    /// Creates a new record.
    pub fn new(fields: Vec<Field<'a>>) -> Self {
        Self(fields)
    }

    /// Decodes a PICA+ encoded record.
    pub fn decode(i: &'a str) -> Result<Self, ParsePicaError> {
        match parse_record(i).finish() {
            Ok((_, record)) => Ok(record),
            _ => Err(ParsePicaError),
        }
    }

    pub fn matches(&self, filter: &Filter) -> bool {
        match filter {
            Filter::FieldExpr(tag, occurrence, filter) => {
                self.iter().any(|field| {
                    field.tag() == tag
                        && field.occurrence() == occurrence.as_deref()
                        && field.matches(filter)
                })
            }
            Filter::BooleanExpr(lhs, op, rhs) => match op {
                BooleanOp::And => self.matches(lhs) && self.matches(rhs),
                BooleanOp::Or => self.matches(lhs) || self.matches(rhs),
            },
            Filter::GroupedExpr(filter) => self.matches(filter),
        }
    }

    /// Returns the record as an PICA3 formatted string.
    pub fn pica3(&self) -> String {
        String::from(
            &*self
                .iter()
                .map(|s| s.pica3())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

impl<'a> Deref for Record<'a> {
    type Target = Vec<Field<'a>>;

    fn deref(&self) -> &Vec<Field<'a>> {
        &self.0
    }
}

/// Parses a PICA+ record, which is a non-empty list of PICA+ fields.
pub fn parse_record(i: &str) -> IResult<&str, Record> {
    map(many1(parse_field), Record::new)(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Subfield;

    #[test]
    fn test_record_new() {
        let record = Record::new(vec![]);
        assert!(record.is_empty());

        let field =
            Field::new("003@", None, vec![Subfield::new('0', "1234567890X")]);
        let record = Record::new(vec![field]);
        assert_eq!(record.pica3(), "003@ $0 1234567890X");
        assert_eq!(record.len(), 1);
    }

    #[test]
    fn test_record_decode() {
        assert_eq!(
            Record::decode("003@ \u{1f}0123456789X\u{1e}").unwrap(),
            Record::new(vec![Field::new(
                "003@",
                None,
                vec![Subfield::new('0', "123456789X")]
            )])
        );

        assert_eq!(Record::decode(""), Err(ParsePicaError));
    }
}
