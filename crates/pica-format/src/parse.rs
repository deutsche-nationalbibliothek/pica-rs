use bstr::ByteSlice;
use pica_matcher::parser::{
    parse_occurrence_matcher, parse_subfield_matcher, parse_tag_matcher,
};
use winnow::ascii::{multispace0, multispace1};
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, separated,
};
use winnow::error::{ContextError, ParserError};
use winnow::prelude::*;
use winnow::stream::{AsChar, Compare, Stream, StreamIsPartial};
use winnow::token::{one_of, take_till};

use crate::{Format, Fragments, Group, List, Value};

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
        parse_list.map(Fragments::List),
        parse_group.map(Fragments::Group),
        parse_value.map(Fragments::Value),
    ))
    .parse_next(i)
}

fn parse_value(i: &mut &[u8]) -> PResult<Value> {
    (opt(ws(parse_string)), parse_codes, opt(ws(parse_string)))
        .map(|(prefix, codes, suffix)| Value {
            prefix,
            codes,
            suffix,
        })
        .parse_next(i)
}

fn parse_group(i: &mut &[u8]) -> PResult<Group> {
    delimited(ws('('), parse_fragments, ws(')'))
        .map(|fragments| Group {
            fragments: Box::new(fragments),
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
fn parse_code(i: &mut &[u8]) -> PResult<char> {
    one_of(('0'..='9', 'a'..='z', 'A'..='Z'))
        .map(char::from)
        .parse_next(i)
}

/// Parses a sequence of subfield codes.
fn parse_codes(i: &mut &[u8]) -> PResult<Vec<char>> {
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
    .map(|s| s.to_str().expect("valid utf8").to_string())
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
