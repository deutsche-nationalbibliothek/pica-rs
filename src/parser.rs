use std::fmt;

use nom::character::complete::char;
use nom::combinator::{all_consuming, opt};
use nom::multi::many1;
use nom::sequence::terminated;

use crate::common::ParseResult;
use crate::field::{parse_field, Field};

const NL: char = '\x0A';

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, PartialEq, Eq)]
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

/// Parses a record.
pub(crate) fn parse_fields(i: &[u8]) -> ParseResult<Vec<Field>> {
    all_consuming(terminated(many1(parse_field), opt(char(NL))))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;
    use crate::{Occurrence, Subfield, Tag};

    #[test]
    fn test_parse_fields() -> TestResult {
        assert_eq!(
            parse_fields(
                b"003@ \x1f0123456789X\x1e012A/00 \x1fa123\x1e012A/01 \x1fa456\x1e"
            )?
            .1,
            vec![
                Field::new(
                    Tag::new("003@")?,
                    None,
                    vec![Subfield::new('0', "123456789X").unwrap()]
                ),
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
}
