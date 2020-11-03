//! Pica+ Parser
//!
//! # Pica+ Grammar
//!
//! ```text
//! Record     ::= Field+
//! Field      ::= Tag Occurrence? ' ' Subfield+ #x1e
//! Tag        ::= [012] [0-9]{2} ([A-Z] | '@')
//! Occurrence ::= '/' [0-9] [0-9]
//! Subfield   ::= #x1f Code Value?
//! Code       ::= [a-zA-Z0-9]
//! Value      ::= [^#x1f#x1e]+
//! ```

use crate::{Field, Path, Record, Subfield};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{none_of, one_of, space0 as space},
    combinator::{all_consuming, map, opt, recognize},
    multi::{count, many0, many1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::fmt;

/// Parse a Pica+ tag.
///
/// A Pica+ tag starts with a digit less than three followed by two digits and
/// an uppercase letter or an '@' character. If the parser succeeds, the
/// remaining input and the tag is returned as an tuple wrapped in an [`Ok`].
pub(self) fn parse_tag(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        one_of("012"),
        count(one_of("0123456789"), 2),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
    )))(i)
}

/// Parse field occurrence.
///
/// The field occurrence is preceded by one '/' character followed by two
/// digits. If the parser succeeds, the remaining input and the occurrence is
/// returned as an tuple wrapped in an [`Ok`].
pub(self) fn parse_occurrence(i: &str) -> IResult<&str, &str> {
    preceded(
        nom::character::complete::char('/'),
        recognize(count(one_of("0123456789"), 2)),
    )(i)
}

/// Parse a subfield code.
///
/// A subfield code is a single character (uppercase, lowercase, digits). The
/// list of codes is sorted by frequency. If the parser succeeds the remaining
/// input and the parsed subfield code is returned.
pub(crate) fn parse_subfield_code(i: &str) -> IResult<&str, char> {
    one_of("a0bd94eA7VSEBHtgDhcfpnmYrK5iuLv6xGjlFqJIoTyzOMP2sRNUX3kZQCw18W")(i)
}

/// Parse a subfield.
///
/// A subfield starts with the unit separator (\x1f) followed by the subfield
/// code (alpha numerical character). The optional subfield value ends with a
/// unit separator or an record separator (\x1f). If the parse succeeds the
/// remaining input and the parsed [`Subfield`] is returned as an tuple wrapped
/// in an [`Ok`].
pub(crate) fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        nom::character::complete::char('\x1f'),
        map(
            pair(parse_subfield_code, recognize(many0(none_of("\x1f\x1e")))),
            |(code, value)| Subfield::new(code, value),
        ),
    )(i)
}

/// Parse a field.
///
/// A field consists of an field tag, a non-empty list of subfields and ends
/// with an record separator (\x1e). If the parser succeeds the remaining input
/// and the parsed [`Field`] is returned as an tuple wrapped in an [`Ok`].
pub(self) fn parse_field(i: &str) -> IResult<&str, Field> {
    terminated(
        map(
            pair(
                pair(parse_tag, opt(parse_occurrence)),
                preceded(
                    nom::character::complete::char(' '),
                    many0(parse_subfield),
                ),
            ),
            |((tag, occurrence), subfields)| {
                Field::new(tag, occurrence, subfields)
            },
        ),
        nom::character::complete::char('\x1e'),
    )(i)
}

/// Parse reccord
///
/// # Example
/// ```
/// use pica::parser::parse_record;
///
/// let (_, record) = parse_record("003@ \x1f0123456789\x1e").unwrap();
/// assert_eq!(record.fields.len(), 1);
/// ```
pub fn parse_record(i: &str) -> IResult<&str, Record> {
    all_consuming(map(many1(parse_field), |fields| Record { fields }))(i)
}

/// Parse a path
///
/// # Example
/// ```
/// use pica::parser::parse_path;
/// let (_, path) = parse_path(" 003@.0   ").unwrap();
/// assert_eq!(path.tag(), "003@");
/// assert_eq!(path.code(), '0');
/// assert!(path.occurrence().is_none());
/// ```
pub fn parse_path(i: &str) -> IResult<&str, Path> {
    all_consuming(map(
        delimited(
            space,
            separated_pair(
                pair(parse_tag, opt(parse_occurrence)),
                nom::character::complete::char('.'),
                parse_subfield_code,
            ),
            space,
        ),
        |((tag, occurrence), code)| Path::new(tag, occurrence, code),
    ))(i)
}

#[derive(Debug, PartialEq)]
pub enum Op {
    Eq,
    Ne,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Op::Eq => write!(f, "=="),
            Op::Ne => write!(f, "!="),
        }
    }
}

/// Parse Op
pub(crate) fn parse_op(i: &str) -> IResult<&str, Op> {
    alt((
        map(delimited(space, tag("=="), space), |_| Op::Eq),
        map(delimited(space, tag("!="), space), |_| Op::Ne),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_op() {
        assert_eq!(parse_op(" == "), Ok(("", Op::Eq)));
        assert_eq!(parse_op(" != "), Ok(("", Op::Ne)));
    }

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path("   003@.0 "),
            Ok(("", Path::new("003@", None, '0')))
        );
        assert_eq!(
            parse_path("   003@/00.0 "),
            Ok(("", Path::new("003@", Some("00"), '0')))
        );
    }

    #[test]
    fn test_parse_tag() {
        assert_eq!(parse_tag("003@"), Ok(("", "003@")));
        assert!(parse_tag("300A").is_err());
        assert!(parse_tag("0A0A").is_err());
        assert!(parse_tag("00AA").is_err());
        assert!(parse_tag("0000").is_err());
    }

    #[test]
    fn test_parse_occurrence() {
        assert_eq!(parse_occurrence("/00"), Ok(("", "00")));
        assert_eq!(parse_occurrence("/01"), Ok(("", "01")));
        assert!(parse_occurrence("00").is_err());
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield("\x1fa123456789"),
            Ok(("", Subfield::new('a', "123456789")))
        );

        assert_eq!(parse_subfield("\x1fa"), Ok(("", Subfield::new('a', ""),)));
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field("003@ \x1f0123456789\x1e"),
            Ok((
                "",
                Field::new("003@", None, vec![Subfield::new('0', "123456789")])
            ))
        );

        assert_eq!(
            parse_field("001B/01 \x1f0123456789\x1e"),
            Ok((
                "",
                Field::new(
                    "001B",
                    Some("01"),
                    vec![Subfield::new('0', "123456789")]
                ),
            ))
        );
    }

    #[test]
    fn test_parse_record() {
        assert_eq!(
            parse_record("003@ \x1f0123456789\x1e"),
            Ok((
                "",
                Record {
                    fields: vec![Field::new(
                        "003@",
                        None,
                        vec![Subfield::new('0', "123456789")]
                    )]
                }
            ))
        );

        assert!(parse_record("003@ \x1f0123456789\x1eabc").is_err())
    }
}
