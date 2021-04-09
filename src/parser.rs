//! This module provides functions to parse PICA+ records.

use crate::{Field, Occurrence, Subfield};

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, one_of, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, recognize, success};
use nom::multi::{count, many0, many1, many_m_n};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::Err;

use bstr::BString;
use std::fmt;

const NL: char = '\x0A';
const US: char = '\x1F';
const RS: char = '\x1E';
const SP: char = '\x20';

/// Parser result.
pub type ParseResult<'a, O> = Result<(&'a [u8], O), Err<()>>;

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, PartialEq)]
pub struct ParsePicaError {
    pub message: String,
    pub data: Vec<u8>,
}

impl std::error::Error for ParsePicaError {}

impl fmt::Display for ParsePicaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// Parses a subfield code.
pub(crate) fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    map(satisfy(|c| c.is_ascii_alphanumeric()), char::from)(i)
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

/// Parses a field occurrence.
pub fn parse_field_occurrence(i: &[u8]) -> ParseResult<Occurrence> {
    map(
        preceded(
            tag(b"/"),
            cut(recognize(many_m_n(2, 3, one_of("0123456789")))),
        ),
        Occurrence::from_unchecked,
    )(i)
}

/// Parses a field tag.
pub fn parse_field_tag(i: &[u8]) -> ParseResult<BString> {
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
pub fn parse_field(i: &[u8]) -> ParseResult<Field> {
    map(
        terminated(
            tuple((
                parse_field_tag,
                alt((map(parse_field_occurrence, Some), success(None))),
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
pub fn parse_fields(i: &[u8]) -> ParseResult<Vec<Field>> {
    all_consuming(terminated(many1(parse_field), opt(char(NL))))(i)
}
