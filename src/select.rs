use crate::field::{parse_field_occurrence, parse_field_tag};
use crate::subfield::parse_subfield_code;
use crate::utils::ws;

use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{cut, map, opt};
use nom::multi::separated_list1;
use nom::sequence::{preceded, terminated, tuple};
use nom::{Finish, IResult};

use std::borrow::Cow;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub struct Selector<'a> {
    pub(crate) tag: Cow<'a, str>,
    pub(crate) occurrence: Option<Cow<'a, str>>,
    pub(crate) subfields: Vec<char>,
}

impl<'a> Selector<'a> {
    pub fn new<S>(tag: S, occurrence: Option<S>, subfields: Vec<char>) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            tag: tag.into(),
            occurrence: occurrence.map(|o| o.into()),
            subfields,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Selectors<'a>(Vec<Selector<'a>>);

impl<'a> Selectors<'a> {
    pub fn parse(i: &'a str) -> Result<Self, String> {
        match parse_selectors(i).finish() {
            Ok((_, result)) => Ok(result),
            Err(_) => Err("invalid selector list".to_string()),
        }
    }
}

impl<'a> Deref for Selectors<'a> {
    type Target = Vec<Selector<'a>>;

    fn deref(&self) -> &Vec<Selector<'a>> {
        &self.0
    }
}

fn parse_single_selector(i: &str) -> IResult<&str, Selector> {
    map(
        tuple((
            parse_field_tag,
            opt(parse_field_occurrence),
            preceded(char('.'), parse_subfield_code),
        )),
        |(tag, occurrence, code)| Selector::new(tag, occurrence, vec![code]),
    )(i)
}

fn parse_multi_selector(i: &str) -> IResult<&str, Selector> {
    map(
        tuple((
            parse_field_tag,
            opt(parse_field_occurrence),
            preceded(
                ws(char('{')),
                cut(terminated(
                    separated_list1(ws(char(',')), parse_subfield_code),
                    ws(char('}')),
                )),
            ),
        )),
        |(tag, occurrence, subfields)| {
            Selector::new(tag, occurrence, subfields)
        },
    )(i)
}

fn parse_selectors(i: &str) -> IResult<&str, Selectors> {
    map(
        separated_list1(
            ws(char(',')),
            alt((parse_single_selector, parse_multi_selector)),
        ),
        Selectors,
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select_columns() {
        assert_eq!(
            parse_selectors("003@.0, 012A/00{a, b, c}"),
            Ok((
                "",
                Selectors(vec![
                    Selector::new("003@", None, vec!['0']),
                    Selector::new("012A", Some("00"), vec!['a', 'b', 'c'])
                ])
            ))
        );
    }
}
