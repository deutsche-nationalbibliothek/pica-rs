//! Pica+ Subfield

use nom::branch::alt;
use nom::character::complete::{char, none_of, one_of};
use nom::combinator::{map, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use nom::IResult;

use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Subfield<'a> {
    pub(crate) code: char,
    pub(crate) value: &'a str,
}

impl<'a> Subfield<'a> {
    pub fn new(code: char, value: &'a str) -> Subfield<'a> {
        Subfield { code, value }
    }

    pub fn code(&self) -> char {
        self.code
    }

    pub fn value(&self) -> &str {
        self.value
    }

    pub fn pretty(&self) -> String {
        format!("${} {}", self.code, self.value)
    }
}

pub(crate) fn parse_subfield_code(i: &str) -> IResult<&str, char> {
    alt((
        one_of("abcdefghijklmnopqrstuvwxyz"),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        one_of("0123456789"),
    ))(i)
}

fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

pub(crate) fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_code, parse_subfield_value),
            |(code, value)| Subfield { code, value },
        ),
    )(i)
}
