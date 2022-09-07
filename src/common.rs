//! Common parser types and functions.

use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::{char, multispace0, multispace1};
use nom::combinator::{map, map_res, value, verify};
use nom::error::ParseError;
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded};
use nom::IResult;

/// Parser result.
pub(crate) type ParseResult<'a, O> =
    Result<(&'a [u8], O), nom::Err<()>>;

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<'a, F: 'a, O, E: ParseError<&'a [u8]>>(
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
