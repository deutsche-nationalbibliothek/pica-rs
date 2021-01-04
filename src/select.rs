use crate::occurrence::{parse_occurrence, Occurrence};
use crate::parser::{parse_field_tag, parse_subfield_name};
use crate::utils::ws;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{cut, map, map_res, opt};
use nom::multi::separated_list1;
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};
use nom::{Finish, IResult};

use std::borrow::Cow;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub enum Range {
    Range(usize, usize),
    RangeFrom(usize),
    RangeTo(usize),
    RangeFull,
}

#[derive(Debug, PartialEq)]
pub struct Selector<'a> {
    pub(crate) tag: Cow<'a, str>,
    pub(crate) occurrence: Occurrence<'a>,
    pub(crate) subfields: Vec<(char, Option<Range>)>,
}

impl<'a> Selector<'a> {
    pub fn new<S>(
        tag: S,
        occurrence: Occurrence<'a>,
        subfields: Vec<(char, Option<Range>)>,
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

fn parse_usize(i: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(i)
}

fn parse_range(i: &str) -> IResult<&str, Range> {
    preceded(
        char('['),
        cut(terminated(
            alt((
                map(
                    separated_pair(parse_usize, tag(".."), parse_usize),
                    |(start, end)| Range::Range(start, end),
                ),
                map(terminated(parse_usize, tag("..")), |start| {
                    Range::RangeFrom(start)
                }),
                map(preceded(tag("..="), parse_usize), |end| {
                    Range::RangeTo(end + 1)
                }),
                map(preceded(tag(".."), parse_usize), |end| {
                    Range::RangeTo(end)
                }),
                map(tag(".."), |_| Range::RangeFull),
                map(parse_usize, |index| Range::Range(index, index + 1)),
            )),
            char(']'),
        )),
    )(i)
}

fn parse_single_selector(i: &str) -> IResult<&str, Selector> {
    map(
        tuple((
            parse_field_tag,
            opt(parse_occurrence),
            preceded(char('.'), pair(parse_subfield_name, opt(parse_range))),
        )),
        |(tag, occurrence, name)| {
            Selector::new(
                tag,
                occurrence.unwrap_or(Occurrence::None),
                vec![name],
            )
        },
    )(i)
}

fn parse_multi_selector(i: &str) -> IResult<&str, Selector> {
    map(
        tuple((
            parse_field_tag,
            opt(parse_occurrence),
            preceded(
                ws(char('{')),
                cut(terminated(
                    separated_list1(
                        ws(char(',')),
                        pair(parse_subfield_name, opt(parse_range)),
                    ),
                    ws(char('}')),
                )),
            ),
        )),
        |(tag, occurrence, subfields)| {
            Selector::new(
                tag,
                occurrence.unwrap_or(Occurrence::None),
                subfields,
            )
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
    fn test_parse_range() {
        assert_eq!(parse_range("[1..1]"), Ok(("", Range::Range(1, 1))));
        assert_eq!(parse_range("[0..1]"), Ok(("", Range::Range(0, 1))));
        assert_eq!(parse_range("[1..]"), Ok(("", Range::RangeFrom(1))));
        assert_eq!(parse_range("[..2]"), Ok(("", Range::RangeTo(2))));
        assert_eq!(parse_range("[..=2]"), Ok(("", Range::RangeTo(3))));
        assert_eq!(parse_range("[..]"), Ok(("", Range::RangeFull)));
        assert_eq!(parse_range("[0]"), Ok(("", Range::Range(0, 1))));
        assert_eq!(parse_range("[1]"), Ok(("", Range::Range(1, 2))));
    }

    #[test]
    fn test_parse_select_columns() {
        assert_eq!(
            parse_selectors("003@.0, 012A/00{a, b, c}"),
            Ok((
                "",
                Selectors(vec![
                    Selector::new("003@", Occurrence::None, vec![('0', None)]),
                    Selector::new(
                        "012A",
                        Occurrence::Value(Cow::Borrowed("00")),
                        vec![('a', None), ('b', None), ('c', None)]
                    )
                ])
            ))
        );
    }
}
