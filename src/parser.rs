use std::fmt;

use nom::branch::alt;
use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, map, opt};
use nom::multi::many1;
use nom::sequence::{delimited, preceded, terminated, tuple};

use crate::common::ParseResult;
use crate::field::{parse_field, Field};
use crate::matcher::{parse_occurrence_matcher, parse_tag_matcher};
use crate::subfield::parse_subfield_code;
use crate::Path;

const NL: char = '\x0A';

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

/// Parses multiple subfield codes.
pub(crate) fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        map(parse_subfield_code, |x| vec![x]),
        delimited(char('['), many1(parse_subfield_code), char(']')),
    ))(i)
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
                parse_tag_matcher,
                parse_occurrence_matcher,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::OccurrenceMatcher;
    use crate::test::TestResult;
    use crate::{Occurrence, Subfield, Tag};

    #[test]
    fn test_parse_fields() -> TestResult {
        assert_eq!(
            parse_fields(b"003@ \x1f0123456789X\x1e012A/00 \x1fa123\x1e012A/01 \x1fa456\x1e")
                ?
                .1,
            vec![
                Field::new(
                    Tag::new("003@")?,
                    None,
                    vec![Subfield::new('0', "123456789X").unwrap()]
                )
                ,
                Field::new(
                    Tag::new("012A")?,
                    None,
                    vec![Subfield::new('a', "123").unwrap()]
                ),
		        Field::new(
                    Tag::new("012A")?,
                    Some(Occurrence::new("01").unwrap()),
                    vec![Subfield::new('a', "456").unwrap()]
                )
            ]
        );

        Ok(())
    }

    #[test]
    fn test_parse_path() -> TestResult {
        assert_eq!(
            parse_path(b"003@.0")?.1,
            Path::new("003@", OccurrenceMatcher::None, vec!['0']).unwrap()
        );
        assert_eq!(
            parse_path(b"012A/01.0")?.1,
            Path::new(
                "012A",
                OccurrenceMatcher::Some(Occurrence::new("01")?),
                vec!['0']
            )
            .unwrap()
        );
        assert_eq!(
            parse_path(b"012A/*.[ab]")?.1,
            Path::new("012A", OccurrenceMatcher::Any, vec!['a', 'b']).unwrap()
        );
        assert_eq!(
            parse_path(b"012A/*.0")?.1,
            Path::new("012A", OccurrenceMatcher::Any, vec!['0']).unwrap()
        );

        Ok(())
    }
}
