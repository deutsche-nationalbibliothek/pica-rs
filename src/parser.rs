use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, one_of, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, recognize, success};
use nom::error::ParseError;
use nom::multi::{many0, many1, many_m_n};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{Err, IResult};

use bstr::BString;

use crate::{Field, Occurrence, Subfield};

pub(crate) type ParseResult<'a, O> = Result<(&'a [u8], O), Err<()>>;

/// Strip whitespaces from the beginning and end
pub(crate) fn ws<'a, O, E: ParseError<&'a [u8]>, F>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parses a PICA+ subfield code.
pub(crate) fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    map(satisfy(|c| c.is_ascii_alphanumeric()), char::from)(i)
}

/// Parses a PICA+ subfield value.
fn parse_subfield_value(i: &[u8]) -> ParseResult<BString> {
    map(recognize(many0(is_not("\x1E\x1F"))), BString::from)(i)
}

// Parses a PICA+ subfield.
fn parse_subfield(i: &[u8]) -> ParseResult<Subfield> {
    map(
        preceded(
            char('\x1f'),
            cut(pair(parse_subfield_code, parse_subfield_value)),
        ),
        |(code, value)| Subfield::from_unchecked(code, value),
    )(i)
}

/// Parses a PICA+ field occurrence.
pub(crate) fn parse_field_occurrence(i: &[u8]) -> ParseResult<Occurrence> {
    map(
        preceded(
            tag(b"/"),
            cut(recognize(many_m_n(2, 3, one_of("0123456789")))),
        ),
        Occurrence::from_unchecked,
    )(i)
}

/// Parses a PICA+ Field tag.
pub(crate) fn parse_field_tag(i: &[u8]) -> ParseResult<BString> {
    map(
        recognize(tuple((
            one_of("012"),
            one_of("0123456789"),
            one_of("0123456789"),
            satisfy(|c| c.is_ascii_uppercase() || c == '@'),
        ))),
        BString::from,
    )(i)
}

/// Parses a PICA+ field.
fn parse_field(i: &[u8]) -> ParseResult<Field> {
    map(
        terminated(
            tuple((
                parse_field_tag,
                alt((map(parse_field_occurrence, Some), success(None))),
                preceded(char(' '), many0(parse_subfield)),
            )),
            char('\x1e'),
        ),
        |(tag, occurrence, subfields)| Field {
            tag,
            occurrence,
            subfields,
        },
    )(i)
}

/// Parse a PICA+ record.
pub(crate) fn parse_fields(i: &[u8]) -> ParseResult<Vec<Field>> {
    all_consuming(terminated(many1(parse_field), opt(char('\n'))))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subfield_code() {
        assert_eq!(parse_subfield_code(b"a").unwrap().1, 'a');
        assert_eq!(parse_subfield_code(b"0").unwrap().1, '0');
        assert!(parse_subfield_code(b"!").is_err());
        assert!(parse_subfield_code(b"").is_err());
    }

    #[test]
    fn test_pase_subfield_value() {
        assert_eq!(parse_subfield_value(b"").unwrap().1, "");
        assert_eq!(parse_subfield_value(b"ab").unwrap().1, "ab");
        assert_eq!(parse_subfield_value(b"a\x1fb").unwrap().1, "a");
        assert_eq!(parse_subfield_value(b"a\x1eb").unwrap().1, "a");
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield(b"\x1fa123").unwrap().1,
            Subfield {
                code: 'a',
                value: BString::from("123")
            }
        );
    }
}
