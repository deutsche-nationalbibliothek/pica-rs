//! This modules contains functions to parse PICA+ records.
//!
//! # Grammar
//!
//! ```text
//! <record>           :: <field>+ NL?
//! <field>            ::= <field-tag> <field-occurrence>? (SP <subfield>*)? RS
//! <field-tag>        ::= [0-2] [0-9]{2} ([A-Z] | '@')
//! <field-occurrence> ::= '/' [0-9]{2,3}
//! <subfield>         ::= US <subfield-code> <subfield-value>?
//! <subfield-code>    ::= [A-Za-z0-9]
//! <subfield-value>   ::= [^\x1e\x1f]
//!
//! <sp> ::= '\x20'
//! <us> ::= '\x1f'
//! <rs> ::= '\x1e'
//! ```

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, one_of, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, recognize, success};
use nom::multi::{many0, many1, many_m_n};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::Err;

use bstr::BString;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct ByteRecord {
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub(crate) tag: BString,
    pub(crate) occurrence: Option<Occurrence>,
    pub(crate) subfields: Vec<Subfield>,
}

impl Field {
    pub fn new<S>(
        tag: S,
        occurrence: Option<Occurrence>,
        subfields: Vec<Subfield>,
    ) -> Self
    where
        S: Into<BString>,
    {
        Self {
            tag: tag.into(),
            occurrence,
            subfields,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Subfield {
    pub(crate) code: char,
    pub(crate) value: BString,
}

impl Subfield {
    /// TODO: eval code and value
    pub fn new<S>(code: char, value: S) -> Self
    where
        S: Into<BString>,
    {
        Self {
            code,
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Occurrence(pub(crate) BString);

impl Occurrence {
    // TODO: eval occurrence
    pub fn new<S>(occurrence: S) -> Self
    where
        S: Into<BString>,
    {
        Self(occurrence.into())
    }
}

#[derive(Debug, PartialEq)]
pub struct ParsePicaError {
    pub message: String,
    pub data: Vec<u8>,
}

impl fmt::Display for ParsePicaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}

pub(crate) type ParseResult<'a, O> = Result<(&'a [u8], O), Err<()>>;

/// Parses a PICA+ subfield code.
fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
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
        |(code, value)| Subfield { code, value },
    )(i)
}

/// Parses a PICA+ field occurrence.
pub fn parse_field_occurrence(i: &[u8]) -> ParseResult<Occurrence> {
    map(
        preceded(
            tag(b"/"),
            cut(recognize(many_m_n(2, 3, one_of("0123456789")))),
        ),
        |value| Occurrence(BString::from(value)),
    )(i)
}

/// Parses a PICA+ Field tag.
pub fn parse_field_tag(i: &[u8]) -> ParseResult<BString> {
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
fn parse_record(i: &[u8]) -> ParseResult<ByteRecord> {
    map(
        all_consuming(terminated(many1(parse_field), opt(char('\n')))),
        |fields| ByteRecord { fields },
    )(i)
}

impl ByteRecord {
    // Creates a new ByteRecord
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    /// Creates a new ByteRecord from a byte vector.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, ParsePicaError> {
        parse_record(&data)
            .map_err(|_| ParsePicaError {
                message: "Invalid record.".to_string(),
                data: data.clone(),
            })
            .map(|(_, record)| record)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_subfield_code() {
        assert_eq!(parse_subfield_code(b"a").unwrap().1, 'a');
        assert!(parse_subfield_code(b"!").is_err());
    }

    #[test]
    fn test_parse_subfield_value() {
        assert_eq!(
            parse_subfield_value(b"foobarbaz").unwrap().1,
            BString::from("foobarbaz")
        );
        assert_eq!(
            parse_subfield_value(b"123\x1fdef").unwrap().1,
            BString::from("123")
        );
        assert_eq!(
            parse_subfield_value(b"123\x1edef").unwrap().1,
            BString::from("123")
        );
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield(b"\x1fa123456789").unwrap().1,
            Subfield::new('a', "123456789")
        )
    }

    #[test]
    fn test_parse_field_occurrence() {
        assert_eq!(
            parse_field_occurrence(b"/01").unwrap().1,
            Occurrence::new("01")
        );
        assert_eq!(
            parse_field_occurrence(b"/001").unwrap().1,
            Occurrence::new("001")
        );
        assert!(parse_field_occurrence(b"/0A").is_err());
    }

    #[test]
    fn test_parse_field_tag() {
        assert_eq!(parse_field_tag(b"003@").unwrap().1, BString::from("003@"));
        assert!(parse_field_tag(b"303@").is_err());
        assert!(parse_field_tag(b"0A3@").is_err());
        assert!(parse_field_tag(b"00A@").is_err());
        assert!(parse_field_tag(b"0000").is_err());
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field(b"012A/00 \x1fa123\x1fb456\x1fc789\x1e")
                .unwrap()
                .1,
            Field::new(
                "012A",
                Some(Occurrence::new("00")),
                vec![
                    Subfield::new('a', "123"),
                    Subfield::new('b', "456"),
                    Subfield::new('c', "789")
                ]
            )
        );
    }

    #[test]
    fn test_parse_record() {
        let record_str =
            b"012A \x1fc789\x1e012A/00 \x1fa123\x1e012A/01 \x1fb456\x1e";
        assert_eq!(
            parse_record(record_str).unwrap().1,
            ByteRecord::new(vec![
                Field::new("012A", None, vec![Subfield::new('c', "789"),]),
                Field::new(
                    "012A",
                    Some(Occurrence::new("00")),
                    vec![Subfield::new('a', "123"),]
                ),
                Field::new(
                    "012A",
                    Some(Occurrence::new("01")),
                    vec![Subfield::new('b', "456"),]
                ),
            ])
        );

        assert_eq!(
            parse_record(b"012A \x1fa123\x1e\n").unwrap().1,
            ByteRecord::new(vec![Field::new(
                "012A",
                None,
                vec![Subfield::new('a', "123"),]
            ),])
        );
    }
}
