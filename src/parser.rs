//! This module provides functions to parse PICA+ records.

use crate::{Field, Occurrence, OccurrenceMatcher, Path, Subfield, Tag};

use std::fmt;
use std::ops::RangeFrom;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, one_of, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, recognize, success};
use nom::error::ParseError;
use nom::multi::{count, many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{AsChar, Err, FindToken, IResult, InputIter, InputLength, Slice};

use bstr::BString;

const NL: char = '\x0A';
const US: char = '\x1F';
const RS: char = '\x1E';
const SP: char = '\x20';

/// Parser result.
pub(crate) type ParseResult<'a, O> = Result<(&'a [u8], O), Err<()>>;

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, PartialEq)]
pub struct ParsePicaError {
    pub message: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct ParsePathError(pub(crate) String);

impl std::error::Error for ParsePicaError {}
impl std::error::Error for ParsePathError {}

impl fmt::Display for ParsePicaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl fmt::Display for ParsePathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Parses a subfield code.
pub(crate) fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    map(satisfy(|c| c.is_ascii_alphanumeric()), char::from)(i)
}

/// Parses multiple subfield codes.
pub(crate) fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        map(parse_subfield_code, |x| vec![x]),
        delimited(char('['), many1(parse_subfield_code), char(']')),
    ))(i)
}

/// Parses a subfield value.
pub(crate) fn parse_subfield_value(i: &[u8]) -> ParseResult<BString> {
    recognize(many0(is_not("\x1E\x1F")))(i).map(|(i, o)| (i, BString::from(o)))
}

/// Parses a subfield.
pub(crate) fn parse_subfield(i: &[u8]) -> ParseResult<Subfield> {
    map(
        preceded(
            char(US),
            cut(pair(parse_subfield_code, parse_subfield_value)),
        ),
        |(code, value)| Subfield::from_unchecked(code, value),
    )(i)
}

/// Parses a field tag.
fn parse_field_tag(i: &[u8]) -> ParseResult<BString> {
    map(
        recognize(tuple((
            one_of("012"),
            count(one_of("0123456789"), 2),
            one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
        ))),
        BString::from,
    )(i)
}

/// Parses a field.
fn parse_field(i: &[u8]) -> ParseResult<Field> {
    map(
        terminated(
            tuple((
                Tag::parse_tag,
                alt((
                    map(tag("/00"), |_| None),
                    map(Occurrence::parse_occurrence, Some),
                    success(None),
                )),
                preceded(char(SP), many0(parse_subfield)),
            )),
            char(RS),
        ),
        |(tag, occurrence, subfields)| Field {
            tag,
            occurrence,
            subfields,
        },
    )(i)
}

/// Parses a record.
pub(crate) fn parse_fields(i: &[u8]) -> ParseResult<Vec<Field>> {
    all_consuming(terminated(many1(parse_field), opt(char(NL))))(i)
}

pub(crate) fn parse_path(i: &[u8]) -> ParseResult<Path> {
    map(
        all_consuming(delimited(
            multispace0,
            tuple((
                parse_field_tag,
                OccurrenceMatcher::parse_occurrence_matcher,
                preceded(char('.'), parse_subfield_codes),
            )),
            multispace0,
        )),
        |(tag, occurrence, codes)| Path {
            tag,
            occurrence,
            codes,
        },
    )(i)
}

pub(crate) fn parse_character_class<I, T, E: ParseError<I>>(
    list: T,
) -> impl FnMut(I) -> IResult<I, Vec<char>, E>
where
    I: Slice<RangeFrom<usize>> + InputIter + Clone + InputLength,
    <I as InputIter>::Item: AsChar + Copy,
    T: FindToken<<I as InputIter>::Item> + Clone,
{
    alt((
        preceded(
            char('['),
            cut(terminated(many1(one_of(list.clone())), char(']'))),
        ),
        map(one_of(list), |x| vec![x]),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subfield_code() {
        assert_eq!(parse_subfield_code(b"0").unwrap().1, '0');
        assert_eq!(parse_subfield_code(b"a").unwrap().1, 'a');
        assert_eq!(parse_subfield_code(b"Z").unwrap().1, 'Z');
        assert!(parse_subfield_code(b"!").is_err());
    }

    #[test]
    fn test_parse_parse_subfield_value() {
        assert_eq!(parse_subfield_value(b"abc").unwrap().1, "abc");
        assert_eq!(parse_subfield_value(b"a\x1ebc").unwrap().1, "a");
        assert_eq!(parse_subfield_value(b"a\x1fbc").unwrap().1, "a");
        assert_eq!(parse_subfield_value(b"").unwrap().1, "");
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield(b"\x1fa123").unwrap().1,
            Subfield::from_unchecked('a', "123")
        );
        assert_eq!(
            parse_subfield(b"\x1fa").unwrap().1,
            Subfield::from_unchecked('a', "")
        );

        assert!(parse_subfield(b"a123").is_err());
        assert!(parse_subfield(b"").is_err());
    }

    #[test]
    fn test_parse_field_tag() {
        assert_eq!(parse_field_tag(b"003@").unwrap().1, BString::from("003@"));
        assert_eq!(parse_field_tag(b"012A").unwrap().1, BString::from("012A"));

        assert!(parse_field_tag(b"003").is_err());
        assert!(parse_field_tag(b"03").is_err());
        assert!(parse_field_tag(b"0").is_err());
        assert!(parse_field_tag(b"").is_err());
        assert!(parse_field_tag(b"003!").is_err());
        assert!(parse_field_tag(b"303@").is_err());
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field(b"003@ \x1f0123456789X\x1e").unwrap().1,
            Field::new(
                Tag::new("003@").unwrap(),
                None,
                vec![Subfield::new('0', "123456789X").unwrap()]
            )
            .unwrap()
        );
    }

    #[test]
    fn test_parse_fields() {
        assert_eq!(
            parse_fields(b"003@ \x1f0123456789X\x1e012A/00 \x1fa123\x1e012A/01 \x1fa456\x1e")
                .unwrap()
                .1,
            vec![
                Field::new(
                    Tag::new("003@").unwrap(),
                    None,
                    vec![Subfield::new('0', "123456789X").unwrap()]
                )
                .unwrap(),
                Field::new(
                    Tag::new("012A").unwrap(),
                    None,
                    vec![Subfield::new('a', "123").unwrap()]
                )
					.unwrap(),
				Field::new(
                    Tag::new("012A").unwrap(),
                    Some(Occurrence::new("01").unwrap()),
                    vec![Subfield::new('a', "456").unwrap()]
                )
                .unwrap()

            ]
        );
    }

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path(b"003@.0").unwrap().1,
            Path::new("003@", OccurrenceMatcher::None, vec!['0']).unwrap()
        );
        assert_eq!(
            parse_path(b"012A/01.0").unwrap().1,
            Path::new("012A", OccurrenceMatcher::new("01").unwrap(), vec!['0'])
                .unwrap()
        );
        assert_eq!(
            parse_path(b"012A/*.[ab]").unwrap().1,
            Path::new("012A", OccurrenceMatcher::Any, vec!['a', 'b']).unwrap()
        );
        assert_eq!(
            parse_path(b"012A/*.0").unwrap().1,
            Path::new("012A", OccurrenceMatcher::Any, vec!['0']).unwrap()
        );
    }
}
