use crate::error::ParsePicaError;
use crate::field::parse_field;
use crate::filter::{BooleanOp, ComparisonOp};
use crate::{Field, Filter, Path};
use nom::combinator::{all_consuming, map};
use nom::multi::many1;
use nom::{Finish, IResult};
use regex::Regex;
use serde::Serialize;
use std::ops::Deref;

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct Record<'a>(Vec<Field<'a>>);

impl<'a> Record<'a> {
    pub fn new(fields: Vec<Field<'a>>) -> Self {
        Self(fields)
    }

    pub fn decode(s: &'a str) -> Result<Self, ParsePicaError> {
        match parse_record(s).finish() {
            Ok((_remaining, record)) => Ok(record),
            _ => Err(ParsePicaError::InvalidRecord),
        }
    }

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
                && field.occurrence() == path.occurrence()
            {
                for subfield in &field.subfields {
                    if subfield.code() == path.code() {
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

    pub fn matches(&self, filter: &Filter) -> bool {
        match filter {
            Filter::ExistenceExpr(path) => !self.path(path).is_empty(),
            Filter::GroupedExpr(filter) => self.matches(filter),
            Filter::NotExpr(filter) => !self.matches(filter),
            Filter::BooleanExpr(lhs, op, rhs) => match op {
                BooleanOp::And => self.matches(lhs) && self.matches(rhs),
                BooleanOp::Or => self.matches(lhs) || self.matches(rhs),
            },
            Filter::ComparisonExpr(path, op, rvalue) => {
                let lvalues = self.path(path);
                match op {
                    ComparisonOp::Eq => {
                        lvalues.into_iter().any(|x| x == rvalue)
                    }
                    ComparisonOp::Ne => {
                        lvalues.into_iter().all(|x| x != rvalue)
                    }
                    ComparisonOp::Re => {
                        let re = Regex::new(rvalue).unwrap();
                        lvalues.into_iter().any(|x| re.is_match(x))
                    }
                    ComparisonOp::StartsWith => {
                        lvalues.into_iter().any(|x| x.starts_with(rvalue))
                    }
                    ComparisonOp::EndsWith => {
                        lvalues.into_iter().any(|x| x.ends_with(rvalue))
                    }
                }
            }
        }
    }
}

impl<'a> Deref for Record<'a> {
    type Target = Vec<Field<'a>>;

    fn deref(&self) -> &Vec<Field<'a>> {
        &self.0
    }
}

fn parse_record(i: &str) -> IResult<&str, Record> {
    all_consuming(map(many1(parse_field), Record::new))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Field, Subfield};

    #[test]
    fn test_parse_record() {
        assert_eq!(
            parse_record("003@ \u{1f}0123456789\u{1e}"),
            Ok((
                "",
                Record::new(vec![Field::new(
                    "003@",
                    None,
                    vec![Subfield::new('0', "123456789").unwrap()]
                )])
            ))
        );
    }
}
