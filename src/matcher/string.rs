use winnow::ascii::multispace1;
use winnow::combinator::{alt, delimited, preceded, repeat};
use winnow::error::{ContextError, ParserError};
use winnow::prelude::*;
use winnow::stream::{AsChar, Compare, Stream, StreamIsPartial};
use winnow::token::take_till;

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
fn parse_string_single_quoted(i: &mut &[u8]) -> PResult<Vec<u8>> {
    parse_quoted_string::<ContextError>(Quotes::Single).parse_next(i)
}

#[inline]
fn parse_string_double_quoted(i: &mut &[u8]) -> PResult<Vec<u8>> {
    parse_quoted_string::<ContextError>(Quotes::Double).parse_next(i)
}

pub(crate) fn parse_string(i: &mut &[u8]) -> PResult<Vec<u8>> {
    alt((parse_string_single_quoted, parse_string_double_quoted))
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
