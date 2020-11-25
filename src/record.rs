//! Pica+ Record

use crate::field::{parse_field, Field};
use nom::combinator::map;
use nom::multi::many1;
use nom::{Finish, IResult};
use serde::Serialize;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsePicaError;

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct Record<'a>(Vec<Field<'a>>);

impl<'a> Record<'a> {
    /// Creates a new record.
    pub fn new(fields: Vec<Field<'a>>) -> Record {
        Record(fields)
    }

    /// Decodes a PICA+ encoded record.
    pub fn decode(i: &'a str) -> Result<Record, ParsePicaError> {
        match parse_record(i).finish() {
            Ok((_, record)) => Ok(record),
            _ => Err(ParsePicaError),
        }
    }

    /// Returns the record as PICA3 formatted string.
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
        let record = Record::decode("003@ \u{1f}0123456789X\u{1e}").unwrap();
        assert_eq!(record.len(), 1);

        assert_eq!(Record::decode(""), Err(ParsePicaError));
    }
}

// use crate::error::ParsePicaError;
// use crate::filter::{BooleanOp, ComparisonOp};
// use crate::parser::parse_record;
// use crate::{Field, Filter, Path};
// use nom::Finish;
// use regex::Regex;
// use serde::Serialize;
// use std::ops::Deref;

// impl Record {

//     pub fn path(&self, path: &Path) -> Vec<&str> {
//         let mut result: Vec<&str> = Vec::new();

//         for field in &self.0 {
//             if field.tag() == path.tag()
//                 && field.occurrence() == path.occurrence()
//             {
//                 for subfield in &field.subfields {
//                     if subfield.code() == path.code() {
//                         result.push(subfield.value());
//                     }
//                 }
//             }
//         }

//         if let Some(index) = path.index() {
//             if let Some(value) = result.get(index) {
//                 return vec![value];
//             } else {
//                 return vec![];
//             }
//         }

//         result
//     }

//     pub fn matches(&self, filter: &Filter) -> bool {
//         match filter {
//             Filter::ExistenceExpr(path) => !self.path(path).is_empty(),
//             Filter::GroupedExpr(filter) => self.matches(filter),
//             Filter::NotExpr(filter) => !self.matches(filter),
//             Filter::BooleanExpr(lhs, op, rhs) => match op {
//                 BooleanOp::And => self.matches(lhs) && self.matches(rhs),
//                 BooleanOp::Or => self.matches(lhs) || self.matches(rhs),
//             },
//             Filter::ComparisonExpr(path, op, rvalue) => {
//                 let lvalues = self.path(path);
//                 match op {
//                     ComparisonOp::Eq => {
//                         lvalues.into_iter().any(|x| x == rvalue)
//                     }
//                     ComparisonOp::Ne => {
//                         lvalues.into_iter().all(|x| x != rvalue)
//                     }
//                     ComparisonOp::Re => {
//                         let re = Regex::new(rvalue).unwrap();
//                         lvalues.into_iter().any(|x| re.is_match(x))
//                     }
//                     ComparisonOp::StartsWith => {
//                         lvalues.into_iter().any(|x| x.starts_with(rvalue))
//                     }
//                     ComparisonOp::EndsWith => {
//                         lvalues.into_iter().any(|x| x.ends_with(rvalue))
//                     }
//                 }
//             }
//         }
//     }
// }

// impl FromStr for Record {
//     type Err = ParsePicaError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match parse_record(s).finish() {
//             Ok((_, record)) => Ok(record),
//             _ => Err(ParsePicaError::InvalidRecord),
//         }
//     }
// }

// impl Deref for Record {
//     type Target = Vec<Field>;

//     fn deref(&self) -> &Vec<Field> {
//         &self.0
//     }
// }
