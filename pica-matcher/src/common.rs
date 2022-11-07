use std::fmt::{self, Display};

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, multispace1};
use nom::combinator::{map, map_res, value, verify};
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded};
use nom::IResult;
use pica_record::parser::ParseResult;

/// Comparison Operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ComparisonOp {
    Eq, // equal, "=="
    Ne, // not equal, "!="
    // Gt,         // greater than, ">"
    // Ge,         // greater than or equal, ">="
    // Lt,         // less than, "<"
    // Le,         // less than or equal, "<="
    StartsWith, // starts with, "=^"
    EndsWith,   // ends with, "=$"
    Similar,    // similar, "=*"
}

impl Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            // Self::Gt => write!(f, ">"),
            // Self::Ge => write!(f, ">="),
            // Self::Lt => write!(f, "<"),
            // Self::Le => write!(f, "<="),
            Self::StartsWith => write!(f, "=^"),
            Self::EndsWith => write!(f, "=$"),
            Self::Similar => write!(f, "=*"),
        }
    }
}

pub(crate) fn parse_comparison_op_str(
    i: &[u8],
) -> ParseResult<ComparisonOp> {
    alt((
        value(ComparisonOp::Eq, tag("==")),
        value(ComparisonOp::Ne, tag("!=")),
        value(ComparisonOp::StartsWith, tag("=^")),
        value(ComparisonOp::EndsWith, tag("=$")),
        value(ComparisonOp::Similar, tag("=*")),
    ))(i)
}

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char(i: &[u8]) -> ParseResult<char> {
    preceded(
        char('\\'),
        alt((
            // parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(i)
}

/// Parse a non-empty block of text that doesn't include \ or ".
fn parse_literal(i: &[u8]) -> ParseResult<&str> {
    map_res(
        verify(is_not("\'\\"), |s: &[u8]| !s.is_empty()),
        std::str::from_utf8,
    )(i)
}

#[derive(Debug, Clone)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWs,
}

/// Combine parse_literal, parse_escaped_char into a StringFragment.
fn parse_fragment(i: &[u8]) -> ParseResult<StringFragment> {
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(
            StringFragment::EscapedWs,
            preceded(char('\\'), multispace1),
        ),
    ))(i)
}

pub(crate) fn parse_string(i: &[u8]) -> ParseResult<String> {
    delimited(
        char('\''),
        fold_many0(
            parse_fragment,
            String::new,
            |mut string, fragment| {
                match fragment {
                    StringFragment::Literal(s) => string.push_str(s),
                    StringFragment::EscapedChar(c) => string.push(c),
                    StringFragment::EscapedWs => {}
                }
                string
            },
        ),
        char('\''),
    )(i)
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_parse_comparison_op_str() {
        assert_done_and_eq!(
            parse_comparison_op_str(b"=="),
            ComparisonOp::Eq
        );
        assert_done_and_eq!(
            parse_comparison_op_str(b"!="),
            ComparisonOp::Ne
        );
        assert_done_and_eq!(
            parse_comparison_op_str(b"=^"),
            ComparisonOp::StartsWith
        );
        assert_done_and_eq!(
            parse_comparison_op_str(b"=$"),
            ComparisonOp::EndsWith
        );
        assert_done_and_eq!(
            parse_comparison_op_str(b"=*"),
            ComparisonOp::Similar
        );
    }

    #[test]
    fn test_parse_string() {
        assert_done_and_eq!(parse_string(b"'abc'"), "abc".to_string());
        assert_done_and_eq!(parse_string(b"'\tc'"), "\tc".to_string());
    }
}
