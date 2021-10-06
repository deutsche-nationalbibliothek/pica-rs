use crate::filter::{
    parse_occurrence_matcher, parse_string, parse_subfield_code,
    parse_subfield_filter, ws, OccurrenceMatcher, SubfieldFilter,
};
use crate::parser::ParseResult;
use crate::TagMatcher;

use nom::branch::alt;
use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, cut, map, opt};
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::Finish;

use bstr::BString;
use std::default::Default;
use std::ops::{Add, Deref, Mul};

#[derive(Debug, PartialEq)]
pub struct FieldSelector {
    pub(crate) tag: TagMatcher,
    pub(crate) occurrence: OccurrenceMatcher,
    pub(crate) filter: Option<SubfieldFilter>,
    pub(crate) subfields: Vec<char>,
}

#[derive(Debug, PartialEq)]
pub enum Selector {
    Field(FieldSelector),
    Value(String),
}

#[derive(Debug, Default)]
pub struct Outcome(pub(crate) Vec<Vec<BString>>);

impl Outcome {
    pub fn from_values(values: Vec<BString>) -> Self {
        Self(vec![values])
    }

    pub fn one() -> Self {
        Self(vec![[BString::from("")].to_vec()])
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for Outcome {
    type Target = Vec<Vec<BString>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Mul for Outcome {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        if self.is_empty() {
            return rhs;
        }

        if rhs.is_empty() {
            return self;
        }

        let mut result: Vec<Vec<BString>> = Vec::new();

        for row_lhs in &self.0 {
            for row in &rhs.0 {
                let mut new_row = row_lhs.clone();
                for col in row {
                    new_row.push(col.to_owned());
                }
                result.push(new_row.clone());
            }
        }

        Self(result)
    }
}

impl Add for Outcome {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result: Vec<Vec<BString>> = Vec::new();

        for row in &self.0 {
            result.push(row.clone())
        }

        for row in &rhs.0 {
            result.push(row.clone())
        }

        Self(result)
    }
}

impl FieldSelector {
    pub fn new(
        tag: TagMatcher,
        occurrence: OccurrenceMatcher,
        filter: Option<SubfieldFilter>,
        subfields: Vec<char>,
    ) -> Self {
        Self {
            tag,
            occurrence,
            filter,
            subfields,
        }
    }
}

pub struct Selectors(Vec<Selector>);

impl Selectors {
    pub fn decode(s: &str) -> Result<Self, String> {
        match parse_selectors(s.as_bytes()).finish() {
            Ok((_, selectors)) => Ok(selectors),
            _ => Err("invalid select statement".to_string()),
        }
    }
}

impl Deref for Selectors {
    type Target = Vec<Selector>;

    fn deref(&self) -> &Vec<Selector> {
        &self.0
    }
}

fn parse_selector(i: &[u8]) -> ParseResult<Selector> {
    alt((
        map(ws(parse_string), Selector::Value),
        map(
            tuple((
                TagMatcher::parse_tag_matcher,
                parse_occurrence_matcher,
                preceded(char('.'), cut(parse_subfield_code)),
            )),
            |(tag_matcher, occurrence, subfield)| {
                Selector::Field(FieldSelector::new(
                    tag_matcher,
                    occurrence,
                    None,
                    vec![subfield],
                ))
            },
        ),
        map(
            tuple((
                TagMatcher::parse_tag_matcher,
                parse_occurrence_matcher,
                delimited(
                    ws(char('{')),
                    pair(
                        opt(terminated(parse_subfield_filter, ws(char(',')))),
                        separated_list1(ws(char(',')), ws(parse_subfield_code)),
                    ),
                    ws(char('}')),
                ),
            )),
            |(tag_matcher, occurrence, (filter, subfields))| {
                Selector::Field(FieldSelector::new(
                    tag_matcher,
                    occurrence,
                    filter,
                    subfields,
                ))
            },
        ),
    ))(i)
}

fn parse_selectors(i: &[u8]) -> ParseResult<Selectors> {
    all_consuming(map(
        delimited(
            multispace0,
            separated_list1(ws(char(',')), parse_selector),
            multispace0,
        ),
        Selectors,
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::ComparisonOp;

    #[test]
    fn test_parse_selector() {
        assert_eq!(
            parse_selector(b"003@.0").unwrap().1,
            Selector::Field(FieldSelector::new(
                TagMatcher::new("003@").unwrap(),
                OccurrenceMatcher::None,
                None,
                vec!['0']
            ))
        );

        assert_eq!(
            parse_selector(b"044H/*{ 9, E ,H}").unwrap().1,
            Selector::Field(FieldSelector::new(
                TagMatcher::new("044H").unwrap(),
                OccurrenceMatcher::Any,
                None,
                vec!['9', 'E', 'H']
            ))
        );

        assert_eq!(
            parse_selector(b"044H/*{ E == 'm', 9, E , H }").unwrap().1,
            Selector::Field(FieldSelector::new(
                TagMatcher::new("044H").unwrap(),
                OccurrenceMatcher::Any,
                Some(SubfieldFilter::Comparison(
                    vec!['E'],
                    ComparisonOp::Eq,
                    vec![BString::from("m")]
                )),
                vec!['9', 'E', 'H']
            ))
        );

        assert_eq!(
            parse_selector(b"012A/*.a").unwrap().1,
            Selector::Field(FieldSelector::new(
                TagMatcher::new("012A").unwrap(),
                OccurrenceMatcher::Any,
                None,
                vec!['a']
            ))
        );

        assert_eq!(
            parse_selector(b"012A/01.a").unwrap().1,
            Selector::Field(FieldSelector::new(
                TagMatcher::new("012A").unwrap(),
                OccurrenceMatcher::new("01").unwrap(),
                None,
                vec!['a']
            ))
        );
    }
}
