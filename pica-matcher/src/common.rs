use std::fmt::{self, Display};

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, multispace1};
use nom::combinator::{map, map_res, value, verify};
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded};
use nom::IResult;
use pica_record::parser::ParseResult;

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Relational Operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RelationalOp {
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

impl Display for RelationalOp {
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

/// Parse RelationalOp which can be used for string comparisons.
pub(crate) fn parse_relational_op_str(
    i: &[u8],
) -> ParseResult<RelationalOp> {
    alt((
        value(RelationalOp::Eq, tag("==")),
        value(RelationalOp::Ne, tag("!=")),
        value(RelationalOp::StartsWith, tag("=^")),
        value(RelationalOp::EndsWith, tag("=$")),
        value(RelationalOp::Similar, tag("=*")),
    ))(i)
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
    fn test_parse_relational_op_str() {
        assert_finished_and_eq!(
            parse_relational_op_str(b"=="),
            RelationalOp::Eq
        );
        assert_finished_and_eq!(
            parse_relational_op_str(b"!="),
            RelationalOp::Ne
        );
        assert_finished_and_eq!(
            parse_relational_op_str(b"=^"),
            RelationalOp::StartsWith
        );
        assert_finished_and_eq!(
            parse_relational_op_str(b"=$"),
            RelationalOp::EndsWith
        );
        assert_finished_and_eq!(
            parse_relational_op_str(b"=*"),
            RelationalOp::Similar
        );
    }

    #[test]
    fn test_parse_string() {
        assert_done_and_eq!(parse_string(b"'abc'"), "abc".to_string());
        assert_done_and_eq!(parse_string(b"'\tc'"), "\tc".to_string());
    }
}
