//! This module contains shared parsers.

use bstr::ByteSlice;
use smallvec::SmallVec;
use winnow::ascii::{multispace0, multispace1};
use winnow::combinator::{
    alt, delimited, preceded, repeat, separated_pair,
};
use winnow::error::{ContextError, ErrMode, ParserError};
use winnow::prelude::*;
use winnow::stream::{AsChar, Compare, Stream, StreamIsPartial};
use winnow::token::take_till;
use winnow::Parser;

use crate::primitives::parse::parse_subfield_code;
use crate::primitives::SubfieldCode;

#[inline]
pub(crate) fn parse_subfield_code_range(
    i: &mut &[u8],
) -> ModalResult<Vec<SubfieldCode>> {
    separated_pair(parse_subfield_code, b'-', parse_subfield_code)
        .verify(|(min, max)| min < max)
        .map(|(min, max)| {
            (min.as_byte()..=max.as_byte())
                .map(SubfieldCode::from_unchecked)
                .collect()
        })
        .parse_next(i)
}

#[inline]
fn parse_subfield_code_list(
    i: &mut &[u8],
) -> ModalResult<Vec<SubfieldCode>> {
    delimited(
        '[',
        repeat(
            1..,
            alt((
                parse_subfield_code_range,
                parse_subfield_code.map(|code| vec![code]),
            )),
        )
        .fold(Vec::new, |mut acc: Vec<_>, item| {
            acc.extend_from_slice(&item);
            acc
        }),
        ']',
    )
    .parse_next(i)
}

#[inline]
fn parse_subfield_code_all(
    i: &mut &[u8],
) -> ModalResult<Vec<SubfieldCode>> {
    const SUBFIELD_CODES: &[u8; 62] = b"0123456789\
        abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    '*'.value(
        SUBFIELD_CODES
            .iter()
            .map(|code| SubfieldCode::from_unchecked(*code))
            .collect(),
    )
    .parse_next(i)
}

/// Parse a list of subfield codes
pub(crate) fn parse_subfield_codes(
    i: &mut &[u8],
) -> ModalResult<SmallVec<[SubfieldCode; 4]>> {
    alt((
        parse_subfield_code_list,
        parse_subfield_code.map(|code| vec![code]),
        parse_subfield_code_all,
    ))
    .map(SmallVec::from_vec)
    .parse_next(i)
}

#[cfg(feature = "compat")]
fn parse_subfield_code_list_compat(
    i: &mut &[u8],
) -> ModalResult<Vec<SubfieldCode>> {
    repeat(1.., parse_subfield_code.map(|code| vec![code]))
        .fold(Vec::new, |mut acc: Vec<_>, item| {
            acc.extend_from_slice(&item);
            acc
        })
        .parse_next(i)
}

#[cfg(feature = "compat")]
pub(crate) fn parse_subfield_codes_compat(
    i: &mut &[u8],
) -> ModalResult<SmallVec<[SubfieldCode; 4]>> {
    alt((parse_subfield_code_list_compat, parse_subfield_code_all))
        .map(SmallVec::from_vec)
        .parse_next(i)
}

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<I, O, E: ParserError<I>, F>(
    mut inner: F,
) -> impl Parser<I, O, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar + Clone,
    F: Parser<I, O, E>,
{
    move |i: &mut I| {
        let _ = multispace0.parse_next(i)?;
        let o = inner.parse_next(i);
        let _ = multispace0.parse_next(i)?;
        o
    }
}

#[derive(Debug, Copy, Clone)]
enum Quotes {
    Single,
    Double,
}

fn parse_literal<I, E>(
    quotes: Quotes,
) -> impl Parser<I, <I as Stream>::Slice, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar,
    E: ParserError<I>,
{
    match quotes {
        Quotes::Single => take_till(1.., ['\'', '\\']),
        Quotes::Double => take_till(1.., ['"', '\\']),
    }
}

fn parse_escaped_char<I, E>(quotes: Quotes) -> impl Parser<I, char, E>
where
    I: Stream + StreamIsPartial + Compare<char>,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I>,
{
    let v = match quotes {
        Quotes::Single => '\'',
        Quotes::Double => '"',
    };

    preceded(
        '\\',
        alt((
            'n'.value('\n'),
            'r'.value('\r'),
            't'.value('\t'),
            'b'.value('\u{08}'),
            'f'.value('\u{0C}'),
            '\\'.value('\\'),
            '/'.value('/'),
            v.value(v),
        )),
    )
}

#[derive(Debug, Clone)]
enum StringFragment<'a> {
    Literal(&'a [u8]),
    EscapedChar(char),
    EscapedWs,
}

fn parse_quoted_fragment<'a, E: ParserError<&'a [u8]>>(
    quotes: Quotes,
) -> impl Parser<&'a [u8], StringFragment<'a>, E> {
    use StringFragment::*;

    alt((
        parse_literal::<&'a [u8], E>(quotes).map(Literal),
        parse_escaped_char::<&'a [u8], E>(quotes).map(EscapedChar),
        preceded('\\', multispace1).value(EscapedWs),
    ))
}

fn parse_quoted_string<'a, E>(
    quotes: Quotes,
) -> impl Parser<&'a [u8], Vec<u8>, E>
where
    E: ParserError<&'a [u8]>,
{
    use StringFragment::*;

    let string_builder = repeat(
        0..,
        parse_quoted_fragment::<E>(quotes),
    )
    .fold(Vec::new, |mut acc, fragment| {
        match fragment {
            Literal(s) => acc.extend_from_slice(s),
            EscapedChar(c) => acc.push(c as u8),
            EscapedWs => {}
        }
        acc
    });

    match quotes {
        Quotes::Single => delimited('\'', string_builder, '\''),
        Quotes::Double => delimited('"', string_builder, '"'),
    }
}

#[inline]
fn parse_string_single_quoted(i: &mut &[u8]) -> ModalResult<Vec<u8>> {
    parse_quoted_string::<ContextError>(Quotes::Single)
        .parse_next(i)
        .map_err(ErrMode::Backtrack)
}

#[inline]
fn parse_string_double_quoted(i: &mut &[u8]) -> ModalResult<Vec<u8>> {
    parse_quoted_string::<ContextError>(Quotes::Double)
        .parse_next(i)
        .map_err(ErrMode::Backtrack)
}

pub(crate) fn parse_string(i: &mut &[u8]) -> ModalResult<Vec<u8>> {
    alt((parse_string_single_quoted, parse_string_double_quoted))
        .verify(|s: &[u8]| s.to_str().is_ok())
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUBFIELD_CODES: &str = "0123456789\
        abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    #[test]
    fn test_parse_subfield_code_range() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_subfield_code_range
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                        .into_iter()
                        .map(SubfieldCode::from_unchecked)
                        .collect::<Vec<_>>()
                );
            };
        }

        parse_success!("a-b", ['a', 'b']);
        parse_success!("a-c", ['a', 'b', 'c']);
        parse_success!("a-z", ('a'..='z'));
        parse_success!("0-9", ('0'..='9'));
        parse_success!("A-Z", ('A'..='Z'));

        assert!(parse_subfield_code_range.parse(b"a-a").is_err());
        assert!(parse_subfield_code_range.parse(b"a-!").is_err());
        assert!(parse_subfield_code_range.parse(b"c-a").is_err());
    }

    #[test]
    fn test_parse_subfield_code_list() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_subfield_code_list
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                        .into_iter()
                        .map(SubfieldCode::from_unchecked)
                        .collect::<Vec<_>>()
                );
            };
        }

        parse_success!("[ab]", ['a', 'b']);
        parse_success!("[abc]", ['a', 'b', 'c']);
        parse_success!("[a-z]", ('a'..='z'));
        parse_success!("[0-9]", ('0'..='9'));
        parse_success!("[A-Z]", ('A'..='Z'));
        parse_success!("[0a-cz]", ['0', 'a', 'b', 'c', 'z']);

        assert!(parse_subfield_code_range.parse(b"[ab!]").is_err());
        assert!(parse_subfield_code_range.parse(b"[a-a]").is_err());
        assert!(parse_subfield_code_range.parse(b"[a-!]").is_err());
        assert!(parse_subfield_code_range.parse(b"[c-a]").is_err());
    }

    #[cfg(feature = "compat")]
    #[test]
    fn test_parse_subfield_code_list_compat() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_subfield_code_list_compat
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                        .into_iter()
                        .map(SubfieldCode::from_unchecked)
                        .collect::<Vec<_>>()
                );
            };
        }

        parse_success!("ab", ['a', 'b']);
        parse_success!("abc", ['a', 'b', 'c']);
        parse_success!("aabc", ['a', 'a', 'b', 'c']);
    }

    #[cfg(feature = "compat")]
    #[test]
    fn test_parse_subfield_code_compat() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_subfield_codes_compat
                        .parse($input.as_bytes())
                        .unwrap(),
                    SmallVec::<[SubfieldCode; 4]>::from_vec(
                        $expected
                            .into_iter()
                            .map(SubfieldCode::from_unchecked)
                            .collect::<Vec<_>>()
                    )
                );
            };
        }

        parse_success!("a", ['a']);
        parse_success!("ab", ['a', 'b']);
        parse_success!("abc", ['a', 'b', 'c']);
    }

    #[test]
    fn test_parse_subfield_code_all() {
        assert_eq!(
            parse_subfield_code_all.parse(b"*").unwrap(),
            SUBFIELD_CODES
                .chars()
                .map(SubfieldCode::from_unchecked)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_parse_string_single_quoted() {
        macro_rules! parse_success {
            ($input:expr, $output:expr) => {
                assert_eq!(
                    parse_string_single_quoted
                        .parse($input.as_bytes())
                        .unwrap(),
                    $output,
                );
            };
        }

        parse_success!("'abc'", b"abc");
        parse_success!("'a\\nbc'", b"a\nbc");
        parse_success!("'a\\rbc'", b"a\rbc");
        parse_success!("'a\\tbc'", b"a\tbc");
        parse_success!("'a\\bbc'", b"a\x08bc");
        parse_success!("'a\\fbc'", b"a\x0Cbc");
        parse_success!("'a\\\\c'", b"a\\c");
        parse_success!("'a\\'c'", b"a\'c");
        parse_success!("'a\"c'", b"a\"c");
        parse_success!("'a\\/bc'", b"a/bc");
        parse_success!("'a\\ bc'", b"abc");
    }

    #[test]
    fn test_parse_string_double_quoted() {
        macro_rules! parse_success {
            ($input:expr, $output:expr) => {
                assert_eq!(
                    parse_string_double_quoted
                        .parse($input.as_bytes())
                        .unwrap(),
                    $output,
                );
            };
        }

        parse_success!("\"abc\"", b"abc");
        parse_success!("\"a\\nbc\"", b"a\nbc");
        parse_success!("\"a\\rbc\"", b"a\rbc");
        parse_success!("\"a\\tbc\"", b"a\tbc");
        parse_success!("\"a\\bbc\"", b"a\x08bc");
        parse_success!("\"a\\fbc\"", b"a\x0Cbc");
        parse_success!("\"a\\\\c\"", b"a\\c");
        parse_success!("\"a'c\"", b"a'c");
        parse_success!("\"a\\/bc\"", b"a/bc");
        parse_success!("\"a\\ bc\"", b"abc");
    }

    mod regressions {
        use super::*;

        /// This bug was found by cargo-fuzz
        #[test]
        fn test_parse_invalid_byte_seq() {
            assert!(parse_string
                .parse(&[
                    39, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                    255, 255, 255, 255, 255, 255, 61, 92, 92, 4, 39,
                ])
                .is_err());
        }
    }
}
