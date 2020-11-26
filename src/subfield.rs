//! Pica+ Subfield

use serde::Serialize;
use std::borrow::Cow;

use nom::character::complete::{char, none_of, satisfy};
use nom::combinator::{map, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use nom::IResult;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Subfield<'a> {
    code: char,
    value: Cow<'a, str>,
}

impl<'a> Subfield<'a> {
    // Creates a new Subfield
    pub fn new<S: Into<Cow<'a, str>>>(code: char, value: S) -> Subfield<'a> {
        Subfield {
            code,
            value: value.into(),
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
    pub fn pica3(&self) -> String {
        format!("${} {}", self.code, self.value)
    }
}

/// Parses a PICA+ subfield code, which is a single alphanumeric ASCII
/// character ([a-zA-Z0-9].
pub(crate) fn parse_subfield_code(i: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

/// Parses a PICA+ subfield value, which is a utf8 string (maybe an empty
/// string) terminated by a record separator (\u{1e}) or an unit separator
/// (\u{1f}).
fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

/// Parses a PICA+ subfield, which consists of an subfield code and an value
/// preceded by an unit separator (\u{1f}).
pub fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_code, parse_subfield_value),
            |(code, value)| Subfield::new(code, value),
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subfield() {
        let subfield = Subfield::new('a', "1234567890");
        assert_eq!(subfield.code(), 'a');
        assert_eq!(subfield.value(), "1234567890");
        assert_eq!(subfield.pica3(), "$a 1234567890");
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield("\u{1f}a1234567890"),
            Ok(("", Subfield::new('a', "1234567890")))
        );

        // empty subfield value
        assert_eq!(parse_subfield("\u{1f}a"), Ok(("", Subfield::new('a', ""))));

        // missing unit separator
        assert!(parse_subfield("a1234567890").is_err());

        // invalid subfield code
        assert!(parse_subfield("\u{1f}!1234567890").is_err());
    }
}
