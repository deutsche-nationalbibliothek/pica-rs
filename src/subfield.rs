//! Pica+ Subfield

use crate::error::ParsePicaError;
use nom::character::complete::{char, none_of, satisfy};
use nom::combinator::{map, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use nom::{Finish, IResult};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Subfield<'a> {
    pub(crate) code: char,
    pub(crate) value: Cow<'a, str>,
}

impl<'a> Subfield<'a> {
    pub fn new<S>(code: char, value: S) -> Result<Self, ParsePicaError>
    where
        S: Into<Cow<'a, str>>,
    {
        if code.is_ascii_alphanumeric() {
            Ok(Subfield {
                code,
                value: value.into(),
            })
        } else {
            Err(ParsePicaError::InvalidSubfield)
        }
    }

    pub fn decode(s: &'a str) -> Result<Self, ParsePicaError> {
        match parse_subfield(s).finish() {
            Ok((_, subfield)) => Ok(subfield),
            _ => Err(ParsePicaError::InvalidSubfield),
        }
    }

    /// Returns the code of the subfield.
    pub fn code(&self) -> char {
        self.code
    }

    // Returns the value of the subfield.
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }

    /// Returns the subfield as an PICA3 encoded string.
    pub fn pretty(&self) -> String {
        format!("${} {}", self.code, self.value)
    }
}

pub(crate) fn parse_subfield_code(i: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

pub(crate) fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_code, parse_subfield_value),
            |(code, value)| Subfield {
                code,
                value: value.into(),
            },
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subfield_code() {
        for range in vec!['a'..='z', 'A'..='Z', '0'..='9'] {
            for c in range {
                assert_eq!(parse_subfield_code(&String::from(c)), Ok(("", c)));
            }
        }
    }

    #[test]
    fn test_parse_subfield_value() {
        assert_eq!(parse_subfield_value(""), Ok(("", "")));
        assert_eq!(parse_subfield_value("abc"), Ok(("", "abc")));
        assert_eq!(parse_subfield_value("ab\u{1f}c"), Ok(("\u{1f}c", "ab")));
        assert_eq!(parse_subfield_value("ab\u{1e}c"), Ok(("\u{1e}c", "ab")));
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield("\u{1f}a123"),
            Ok(("", Subfield::new('a', "123").unwrap()))
        );
        assert!(parse_subfield("!a123").is_err());
    }
}
