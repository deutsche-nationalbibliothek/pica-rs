//! This module contains functions to parse PICA+ encoded records.
//!
//! A PICA+ Record is defined as followed:
//!
//! ```text
//! <record>         := <field>+
//! <field>          := <field-name> <occurrence>? <sp> <subfield>+ <rs>
//! <field-name>     := [0-2] [0-9]{2} ([A-Z] | "@")
//! <occurrence>     := "/" [0-9]{2,3}
//! <subfield>       := <us> <subfield-name> <subfield-value>
//! <subfield-name>  := [0-9A-Za-z]
//! <subfield-value> := [^<us><rs>]*
//!
//! <sp> := #x20
//! <us> := #x1f
//! <rs> := #x1e
//! ```

use crate::subfield::Subfield;

use nom::character::complete::{char, none_of, satisfy};
use nom::combinator::{map, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use nom::IResult;

/// Parses a subfield name
pub(crate) fn parse_subfield_name(i: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

/// Parses a subfield value
fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

// Parses a subfield
pub(crate) fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_name, parse_subfield_value),
            |(name, value)| Subfield {
                name,
                value: value.into(),
            },
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subfield_name() {
        for range in vec!['a'..='z', 'A'..='Z', '0'..='9'] {
            for c in range {
                assert_eq!(parse_subfield_name(&String::from(c)), Ok(("", c)));
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
