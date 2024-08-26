use winnow::ascii::{multispace0, multispace1};
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, separated,
};
use winnow::error::{ContextError, ParserError};
use winnow::prelude::*;
use winnow::stream::{AsChar, Compare, Stream, StreamIsPartial};
use winnow::token::{one_of, take_till};

use crate::{Atom, Format, Fragment, Group};

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

/// Parses a subfield code (a single alpha-numeric character)
fn parse_subfield_code(i: &mut &str) -> PResult<char> {
    one_of(('0'..='9', 'a'..='z', 'A'..='Z')).parse_next(i)
}

/// Parses a sequence of subfield codes.
fn parse_subfield_codes(i: &mut &str) -> PResult<Vec<char>> {
    alt((
        parse_subfield_code.map(|code| vec![code]),
        delimited(ws('['), repeat(1.., parse_subfield_code), ws(']')),
    ))
    .parse_next(i)
}

#[derive(Debug, Clone)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWs,
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

fn parse_quoted_string_fragment<'a, E: ParserError<&'a str>>(
    quotes: Quotes,
) -> impl Parser<&'a str, StringFragment<'a>, E> {
    use StringFragment::*;

    alt((
        parse_literal::<&'a str, E>(quotes).map(Literal),
        parse_escaped_char::<&'a str, E>(quotes).map(EscapedChar),
        preceded('\\', multispace1).value(EscapedWs),
    ))
}

fn parse_quoted_string<'a, E>(
    quotes: Quotes,
) -> impl Parser<&'a str, String, E>
where
    E: ParserError<&'a str>,
{
    use StringFragment::*;

    let builder = repeat(
        0..,
        parse_quoted_string_fragment::<E>(quotes),
    )
    .fold(String::new, |mut acc, fragment| {
        match fragment {
            Literal(s) => acc.push_str(s),
            EscapedChar(c) => acc.push(c),
            EscapedWs => {}
        }

        acc
    });

    match quotes {
        Quotes::Single => delimited('\'', builder, '\''),
        Quotes::Double => delimited('"', builder, '"'),
    }
}

fn parse_single_quoted_string(i: &mut &str) -> PResult<String> {
    parse_quoted_string::<ContextError>(Quotes::Single).parse_next(i)
}

fn parse_double_quoted_string(i: &mut &str) -> PResult<String> {
    parse_quoted_string::<ContextError>(Quotes::Double).parse_next(i)
}

fn parse_string(i: &mut &str) -> PResult<String> {
    alt((parse_single_quoted_string, parse_double_quoted_string))
        .parse_next(i)
}

fn parse_atom(i: &mut &str) -> PResult<Atom> {
    (
        opt(ws(parse_string)),
        ws(parse_subfield_codes),
        opt(ws(parse_string)),
    )
        .map(|(prefix, codes, suffix)| Atom {
            prefix,
            codes,
            suffix,
        })
        .parse_next(i)
}

fn parse_group(i: &mut &str) -> PResult<Group> {
    delimited(
        ws('('),
        ws(separated(1.., parse_atom, ws("<|>"))
            .map(|atoms| Group { atoms })),
        ws(')'),
    )
    .parse_next(i)
}

/// Parses a format fragment.
fn parse_fragment(i: &mut &str) -> PResult<Fragment> {
    alt((
        ws(parse_group).map(Fragment::Group),
        ws(parse_atom).map(Fragment::Atom),
    ))
    .parse_next(i)
}

/// Parses a format string.
pub(crate) fn parse_format(i: &mut &str) -> PResult<Format> {
    repeat(1.., parse_fragment).map(Format).parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subfield_code() {
        for c in '\0'..=char::MAX {
            if c.is_ascii_alphanumeric() {
                assert_eq!(
                    parse_subfield_codes.parse(&format!("[{c}]")),
                    Ok(vec![c])
                );
                assert_eq!(
                    parse_subfield_codes.parse(&format!("{c}")),
                    Ok(vec![c])
                );
            } else {
                assert!(parse_subfield_codes
                    .parse(&format!("$[{c}]"))
                    .is_err());
                assert!(parse_subfield_codes
                    .parse(&format!("${c}"))
                    .is_err());
            }
        }
    }
}
