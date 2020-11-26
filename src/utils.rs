use nom::branch::alt;
use nom::bytes::streaming::{is_not, take_while_m_n};
use nom::character::complete::{char, multispace0, multispace1};
use nom::combinator::{map, map_opt, map_res, value, verify};
use nom::error::{FromExternalError, ParseError};
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded};
use nom::IResult;

/// Strip whitespaces from the beginning and end.
pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1-6 hex
/// numerals. We will combine this later with parse_escaped_char to parse
/// sequences like \u{00AC}.
fn parse_unicode<'a, E>(i: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let parse_delimited_hex = preceded(
        char('u'),
        delimited(
            char('{'),
            take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit()),
            char('}'),
        ),
    );

    map_opt(
        map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16)),
        std::char::from_u32,
    )(i)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char<'a, E>(i: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        alt((
            parse_unicode,
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
fn parse_literal<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    verify(is_not("\'\\"), |s: &str| !s.is_empty())(i)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a, E>(i: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, preceded(char('\\'), multispace1)),
    ))(i)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
pub(crate) fn parse_string<'a, E>(i: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    delimited(
        char('\''),
        fold_many0(parse_fragment, String::new(), |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        }),
        char('\''),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_helper(i: &str) -> IResult<&str, String> {
        parse_string(i)
    }

    #[test]
    fn test_parse_unicode() {
        assert_eq!(
            parse_helper("'\u{1f}\t'"),
            Ok(("", String::from("\x1f\t")))
        );
    }
}
