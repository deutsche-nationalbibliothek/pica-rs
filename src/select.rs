use crate::filter::{
    parse_field_tag, parse_occurrence_matcher, parse_subfield_code, ws,
    OccurrenceMatcher,
};

use nom::branch::alt;
use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, cut, map};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, tuple};
use nom::{Finish, IResult};

use bstr::{BStr, ByteSlice};
use std::default::Default;
use std::ops::{Add, Deref, Mul};

#[derive(Debug, PartialEq)]
pub struct Selector {
    pub(crate) tag: String,
    pub(crate) occurrence: OccurrenceMatcher,
    pub(crate) subfields: Vec<char>,
}

#[derive(Debug)]
pub struct Outcome<'a>(pub(crate) Vec<Vec<&'a BStr>>);

impl<'a> Outcome<'a> {
    pub fn from_values(values: Vec<&'a BStr>) -> Self {
        Self(vec![values])
    }

    pub fn one() -> Self {
        Self(vec![[b"".as_bstr()].to_vec()])
    }
}

impl<'a> Default for Outcome<'a> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<'a> Deref for Outcome<'a> {
    type Target = Vec<Vec<&'a BStr>>;

    fn deref(&self) -> &Vec<Vec<&'a BStr>> {
        &self.0
    }
}

impl<'a> Mul for Outcome<'a> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        if self.is_empty() {
            return rhs;
        }

        if rhs.is_empty() {
            return self;
        }

        let mut result: Vec<Vec<&'a BStr>> = Vec::new();

        for row_lhs in &self.0 {
            for row in &rhs.0 {
                let mut new_row = row_lhs.clone();
                for col in row {
                    new_row.push(col);
                }
                result.push(new_row.clone());
            }
        }

        Self(result)
    }
}

impl<'a> Add for Outcome<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result: Vec<Vec<&'a BStr>> = Vec::new();

        for row in &self.0 {
            result.push(row.clone())
        }

        for row in &rhs.0 {
            result.push(row.clone())
        }

        Self(result)
    }
}

impl Selector {
    pub fn new(
        tag: String,
        occurrence: OccurrenceMatcher,
        subfields: Vec<char>,
    ) -> Self {
        Self {
            tag,
            occurrence,
            subfields,
        }
    }
}

pub struct Selectors(Vec<Selector>);

impl Selectors {
    pub fn decode(s: &str) -> Result<Self, String> {
        match parse_selectors(s).finish() {
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

fn parse_selector(i: &str) -> IResult<&str, Selector> {
    map(
        tuple((
            parse_field_tag,
            parse_occurrence_matcher,
            alt((
                // single subfield
                map(
                    preceded(char('.'), cut(parse_subfield_code)),
                    |subfield| vec![subfield],
                ),
                // multiple subfields
                delimited(
                    ws(char('{')),
                    separated_list1(ws(char(',')), ws(parse_subfield_code)),
                    ws(char('}')),
                ),
            )),
        )),
        |(tag, occurrence, subfields)| {
            Selector::new(String::from(tag), occurrence, subfields)
        },
    )(i)
}

fn parse_selectors(i: &str) -> IResult<&str, Selectors> {
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

    #[test]
    fn test_parse_selector() {
        assert_eq!(
            parse_selector("003@.0"),
            Ok((
                "",
                Selector::new(
                    "003@".to_string(),
                    OccurrenceMatcher::None,
                    vec!['0']
                )
            ))
        );

        assert_eq!(
            parse_selector("044H/*{ 9, E ,H}"),
            Ok((
                "",
                Selector::new(
                    "044H".to_string(),
                    OccurrenceMatcher::Ignore,
                    vec!['9', 'E', 'H']
                )
            ))
        );

        assert_eq!(
            parse_selector("012A/*.a"),
            Ok((
                "",
                Selector::new(
                    "012A".to_string(),
                    OccurrenceMatcher::Ignore,
                    vec!['a']
                )
            ))
        );

        assert_eq!(
            parse_selector("012A/01.a"),
            Ok((
                "",
                Selector::new(
                    "012A".to_string(),
                    OccurrenceMatcher::Value("01".to_string()),
                    vec!['a']
                )
            ))
        );
    }
}
