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

use crate::{
    legacy::{Record, Subfield},
    Field, Occurrence,
};

use nom::character::complete::{char, none_of, one_of, satisfy};
use nom::combinator::{map, opt, recognize};
use nom::multi::{count, many0, many1, many_m_n};
use nom::sequence::{pair, preceded, terminated, tuple};
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

pub(crate) fn parse_field_tag(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        one_of("012"),
        count(one_of("0123456789"), 2),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
    )))(i)
}

pub(crate) fn parse_field_occurrence(i: &str) -> IResult<&str, Occurrence> {
    preceded(
        char('/'),
        map(
            recognize(many_m_n(2, 3, one_of("0123456789"))),
            Occurrence::new,
        ),
    )(i)
}

pub(crate) fn parse_field(i: &str) -> IResult<&str, Field> {
    terminated(
        map(
            pair(
                pair(parse_field_tag, opt(parse_field_occurrence)),
                preceded(char(' '), many0(parse_subfield)),
            ),
            |((tag, occurrence), subfields)| {
                Field::new(tag, occurrence, subfields)
            },
        ),
        char('\u{1e}'),
    )(i)
}

pub(crate) fn parse_record(i: &str) -> IResult<&str, Record> {
    map(many1(parse_field), Record::new)(i)
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

    #[test]
    fn test_parse_field_tag() {
        for tag in vec!["000A", "100A", "200A", "000A", "000@"] {
            assert_eq!(parse_field_tag(tag), Ok(("", tag)));
        }

        for tag in vec!["300A", "0A0A", "00AA", "0001"] {
            assert!(parse_field_tag(tag).is_err())
        }
    }

    #[test]
    fn test_parse_field_occurrence() {
        assert_eq!(
            parse_field_occurrence("/00"),
            Ok(("", Occurrence::new("00")))
        );
        assert_eq!(
            parse_field_occurrence("/001"),
            Ok(("", Occurrence::new("001")))
        );
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field("012A/00 \u{1e}"),
            Ok(("", Field::new("012A", Some(Occurrence::new("00")), vec![])))
        );
        assert_eq!(
            parse_field("012A \u{1e}"),
            Ok(("", Field::new("012A", None, vec![])))
        );
        assert_eq!(
            parse_field("003@ \u{1f}0123456789\u{1e}"),
            Ok((
                "",
                Field::new(
                    "003@",
                    None,
                    vec![Subfield::new('0', "123456789").unwrap()]
                )
            ))
        );
    }

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
