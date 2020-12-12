use crate::error::ParsePicaError;
use crate::field::parse_field;
use crate::filter::BooleanOp;
use crate::select::{Range, Selector, Selectors};
use crate::Filter;
use crate::{Field, Path};
use nom::combinator::{all_consuming, map};
use nom::multi::many1;
use nom::{Finish, IResult};
use serde::Serialize;
use std::borrow::Cow;
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

    fn collect(&self, selector: &Selector) -> Vec<Vec<String>> {
        let mut retval: Vec<Vec<String>> = Vec::new();

        for field in self.iter() {
            if field.tag == selector.tag
                && field.occurrence == selector.occurrence
            {
                let mut temp = vec![];
                for (code, range) in &selector.subfields {
                    let mut values: Vec<Cow<'_, str>> = field
                        .subfields()
                        .iter()
                        .filter(|subfield| subfield.code == *code)
                        .map(|subfield| subfield.value.clone())
                        .collect::<Vec<_>>();

                    let values_ranged = if let Some(range) = range {
                        match range {
                            Range::Range(start, end) => &values[*start..*end],
                            Range::RangeTo(start) => &values[*start..],
                            Range::RangeFrom(end) => &values[..*end],
                            Range::RangeFull => &values[..],
                        }
                    } else {
                        &values[..]
                    };

                    values = values_ranged.to_vec();

                    if values.is_empty() {
                        values.push(Cow::Borrowed(""))
                    }

                    temp.push(values);
                }

                let mut result = temp.iter().fold(vec![vec![]], |acc, x| {
                    let mut tmp: Vec<Vec<String>> = vec![];

                    for item in x {
                        for row in &acc {
                            let mut new_row: Vec<String> = row.clone();
                            new_row.push(String::from(item.clone()));
                            tmp.push(new_row);
                        }
                    }

                    tmp
                });

                retval.append(&mut result);
            }
        }

        if retval.is_empty() {
            retval.push(
                selector
                    .subfields
                    .iter()
                    .map(|_| "".to_string())
                    .collect::<Vec<_>>(),
            )
        }

        retval
    }

    pub fn select(&self, selectors: &Selectors) -> Vec<Vec<String>> {
        let result = selectors
            .iter()
            .map(|selector| self.collect(&selector))
            .fold(vec![vec![]], |acc, mut x| {
                if x.is_empty() {
                    x = vec![vec!["".to_string()]];
                }

                let mut tmp: Vec<Vec<String>> = vec![];
                for item in x {
                    for row in &acc {
                        let mut new_row: Vec<String> = row.clone();
                        new_row.append(&mut item.clone());
                        tmp.push(new_row.clone());
                    }
                }

                tmp
            });

        result
    }

    pub fn matches(&self, filter: &Filter) -> bool {
        match filter {
            Filter::Field(tag, occurrence, filter) => {
                self.iter().any(|field| {
                    field.tag() == tag
                        && field.occurrence() == occurrence.as_deref()
                        && field.matches(filter)
                })
            }
            Filter::Boolean(lhs, op, rhs) => match op {
                BooleanOp::And => self.matches(lhs) && self.matches(rhs),
                BooleanOp::Or => self.matches(lhs) || self.matches(rhs),
            },
            Filter::Grouped(filter) => self.matches(filter),
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
