use crate::legacy::{
    parse_field_tag, parse_occurrence_matcher, parse_subfield_name, ws,
    OccurrenceMatcher,
};

use nom::branch::alt;
use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, cut, map};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, tuple};
use nom::{Finish, IResult};

use std::borrow::Cow;
use std::default::Default;
use std::ops::{Add, Deref, Mul};

#[derive(Debug, PartialEq)]
pub struct Selector<'a> {
    pub(crate) tag: Cow<'a, str>,
    pub(crate) occurrence: OccurrenceMatcher<'a>,
    pub(crate) subfields: Vec<char>,
}

#[derive(Debug)]
pub struct Outcome<'a>(pub(crate) Vec<Vec<&'a str>>);

impl<'a> Outcome<'a> {
    pub fn from_values(values: Vec<&'a str>) -> Self {
        Self(vec![values])
    }

    pub fn one() -> Self {
        Self(vec![[""].to_vec()])
    }
}

impl<'a> Default for Outcome<'a> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<'a> Deref for Outcome<'a> {
    type Target = Vec<Vec<&'a str>>;

    fn deref(&self) -> &Vec<Vec<&'a str>> {
        &self.0
    }
}

impl<'a> Mul for Outcome<'a> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        // println!("MUL: lhs = {:?}, rhs = {:?}", self, rhs);

        if self.is_empty() {
            return rhs;
        }

        if rhs.is_empty() {
            return self;
        }

        let mut result: Vec<Vec<&'a str>> = Vec::new();

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
        // println!("ADD: lhs = {:?}, rhs = {:?}", self, rhs);

        let mut result: Vec<Vec<&'a str>> = Vec::new();

        for row in &self.0 {
            result.push(row.clone())
        }

        for row in &rhs.0 {
            result.push(row.clone())
        }

        Self(result)
    }
}

impl<'a> Selector<'a> {
    pub fn new<S>(
        tag: S,
        occurrence: OccurrenceMatcher<'a>,
        subfields: Vec<char>,
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
}

pub struct Selectors<'a>(Vec<Selector<'a>>);

impl<'a> Selectors<'a> {
    pub fn decode(s: &'a str) -> Result<Self, String> {
        match parse_selectors(s).finish() {
            Ok((_, selectors)) => Ok(selectors),
            _ => Err("invalid select statement".to_string()),
        }
    }
}

impl<'a> Deref for Selectors<'a> {
    type Target = Vec<Selector<'a>>;

    fn deref(&self) -> &Vec<Selector<'a>> {
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
                    preceded(char('.'), cut(parse_subfield_name)),
                    |subfield| vec![subfield],
                ),
                // multiple subfields
                delimited(
                    ws(char('{')),
                    separated_list1(ws(char(',')), ws(parse_subfield_name)),
                    ws(char('}')),
                ),
            )),
        )),
        |(tag, occurrence, subfields)| {
            Selector::new(tag, occurrence, subfields)
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
                Selector::new("003@", OccurrenceMatcher::None, vec!['0'])
            ))
        );

        assert_eq!(
            parse_selector("044H/*{ 9, E ,H}"),
            Ok((
                "",
                Selector::new(
                    "044H",
                    OccurrenceMatcher::All,
                    vec!['9', 'E', 'H']
                )
            ))
        );

        assert_eq!(
            parse_selector("012A/*.a"),
            Ok(("", Selector::new("012A", OccurrenceMatcher::All, vec!['a'])))
        );

        assert_eq!(
            parse_selector("012A/01.a"),
            Ok((
                "",
                Selector::new(
                    "012A",
                    OccurrenceMatcher::value("01"),
                    vec!['a']
                )
            ))
        );
    }
}
