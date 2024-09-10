use std::cell::RefCell;
use std::ops::RangeTo;

use bstr::ByteSlice;
use pica_matcher::parser::{
    parse_occurrence_matcher, parse_subfield_matcher, parse_tag_matcher,
};
use pica_record_v1::SubfieldCode;
use winnow::ascii::{digit1, multispace0, multispace1};
use winnow::combinator::{
    alt, delimited, empty, opt, preceded, repeat, separated, terminated,
};
use winnow::error::{ContextError, ParserError};
use winnow::prelude::*;
use winnow::stream::{AsChar, Compare, Stream, StreamIsPartial};
use winnow::token::{one_of, take_till};

use crate::{Format, Fragments, Group, List, Modifier, Value};

pub fn parse_format(i: &mut &[u8]) -> PResult<Format> {
    (
        parse_tag_matcher,
        parse_occurrence_matcher,
        delimited(
            ws('{'),
            (
                parse_fragments,
                opt(preceded(ws('|'), parse_subfield_matcher)),
            ),
            ws('}'),
        ),
    )
        .map(|(t, o, (f, s))| Format {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: s,
            fragments: f,
        })
        .parse_next(i)
}

fn parse_fragments(i: &mut &[u8]) -> PResult<Fragments> {
    alt((
        ws(parse_list).map(Fragments::List),
        ws(parse_group).map(Fragments::Group),
        ws(parse_value).map(Fragments::Value),
    ))
    .parse_next(i)
}

fn parse_value(i: &mut &[u8]) -> PResult<Value> {
    (
        opt(ws(parse_string)),
        parse_codes,
        alt((
            preceded(
                "..",
                digit1
                    .verify_map(|s: &[u8]| s.to_str().ok())
                    .verify_map(|s: &str| s.parse::<usize>().ok()),
            ),
            "..".value(usize::MAX),
            empty.value(1),
        )),
        opt(ws(parse_string)),
    )
        .map(|(prefix, codes, end, suffix)| Value {
            prefix,
            codes,
            suffix,
            bounds: RangeTo { end },
        })
        .parse_next(i)
}

thread_local! {
    pub static GROUP_LEVEL: RefCell<u32> = const { RefCell::new(0) };
}

fn increment_group_level(i: &mut &[u8]) -> PResult<()> {
    GROUP_LEVEL.with(|level| {
        *level.borrow_mut() += 1;
        if *level.borrow() >= 32 {
            Err(winnow::error::ErrMode::from_error_kind(
                i,
                winnow::error::ErrorKind::Many,
            ))
        } else {
            Ok(())
        }
    })
}

fn decrement_group_level() {
    GROUP_LEVEL.with(|level| {
        *level.borrow_mut() -= 1;
    })
}

fn parse_modifier(i: &mut &[u8]) -> PResult<Option<Modifier>> {
    opt(preceded(
        '?',
        repeat(1.., alt(('L', 'U', 'T', 'W'))).map(|codes: Vec<_>| {
            let mut modifier = Modifier::default();
            if codes.contains(&'L') {
                modifier.lowercase(true);
            }

            if codes.contains(&'U') {
                modifier.uppercase(true);
            }

            if codes.contains(&'W') {
                modifier.remove_ws(true);
            }

            if codes.contains(&'T') {
                modifier.trim(true);
            }

            modifier
        }),
    ))
    .parse_next(i)
}

fn parse_group(i: &mut &[u8]) -> PResult<Group> {
    (
        terminated(ws('('), increment_group_level),
        parse_modifier,
        parse_fragments,
        ws(')').map(|_| decrement_group_level()),
        alt((
            preceded(
                "..",
                digit1
                    .verify_map(|s: &[u8]| s.to_str().ok())
                    .verify_map(|s: &str| s.parse::<usize>().ok()),
            ),
            "..".value(usize::MAX),
            empty.value(usize::MAX),
        )),
    )
        .map(|(_, modifier, fragments, _, end)| Group {
            fragments: Box::new(fragments),
            bounds: RangeTo { end },
            modifier: modifier.unwrap_or_default(),
        })
        .parse_next(i)
}

fn parse_list(i: &mut &[u8]) -> PResult<List> {
    alt((parse_list_cons, parse_list_and_then)).parse_next(i)
}

fn parse_list_cons(i: &mut &[u8]) -> PResult<List> {
    separated(
        2..,
        alt((
            parse_list_and_then.map(Fragments::List),
            parse_group.map(Fragments::Group),
            parse_value.map(Fragments::Value),
        )),
        ws("<*>"),
    )
    .map(List::Cons)
    .parse_next(i)
}

fn parse_list_and_then(i: &mut &[u8]) -> PResult<List> {
    separated(
        2..,
        alt((
            parse_group.map(Fragments::Group),
            parse_value.map(Fragments::Value),
        )),
        ws("<$>"),
    )
    .map(List::AndThen)
    .parse_next(i)
}

/// Parses a subfield code (a single alpha-numeric character)
fn parse_code(i: &mut &[u8]) -> PResult<SubfieldCode> {
    one_of(('0'..='9', 'a'..='z', 'A'..='Z'))
        .map(SubfieldCode::from_unchecked)
        .parse_next(i)
}

/// Parses a sequence of subfield codes.
fn parse_codes(i: &mut &[u8]) -> PResult<Vec<SubfieldCode>> {
    alt((
        parse_code.map(|code| vec![code]),
        delimited(ws('['), repeat(2.., parse_code), ws(']')),
    ))
    .parse_next(i)
}

fn parse_string(i: &mut &[u8]) -> PResult<String> {
    alt((
        parse_quoted_string::<ContextError>(Quotes::Single),
        parse_quoted_string::<ContextError>(Quotes::Double),
    ))
    .verify_map(|s| s.to_str().map(ToString::to_string).ok())
    .parse_next(i)
}

#[derive(Debug, Copy, Clone)]
enum Quotes {
    Single,
    Double,
}

#[derive(Debug, Clone)]
enum StringFragment<'a> {
    Literal(&'a [u8]),
    EscapedChar(char),
    EscapedWs,
}

fn parse_quoted_string<'a, E>(
    quotes: Quotes,
) -> impl Parser<&'a [u8], Vec<u8>, E>
where
    E: ParserError<&'a [u8]>,
{
    use StringFragment::*;

    let builder = repeat(
        0..,
        parse_quoted_string_fragment::<E>(quotes),
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
        Quotes::Single => delimited('\'', builder, '\''),
        Quotes::Double => delimited('"', builder, '"'),
    }
}

fn parse_quoted_string_fragment<'a, E: ParserError<&'a [u8]>>(
    quotes: Quotes,
) -> impl Parser<&'a [u8], StringFragment<'a>, E> {
    use StringFragment::*;

    alt((
        parse_literal::<&'a [u8], E>(quotes).map(Literal),
        parse_escaped_char::<&'a [u8], E>(quotes).map(EscapedChar),
        preceded('\\', multispace1).value(EscapedWs),
    ))
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

/// Strip whitespaces from the beginning and end.
fn ws<I, O, E: ParserError<I>, F>(mut inner: F) -> impl Parser<I, O, E>
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

#[cfg(test)]
mod tests {
    use super::*;

    mod regressions {
        use super::*;

        /// This bug was found by cargo-fuzz. For the complete data see
        /// crash-1065da7d802c4cec5ff86325a5629a0e4736191d inside the
        /// crates/pica-select/fuzz/regressions/ directory.
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
