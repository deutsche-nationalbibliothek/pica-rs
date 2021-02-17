//! This module provides functions to parse PICA+ records.

use bstr::{BStr, ByteSlice};

use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, satisfy};
use nom::combinator::{cut, map, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use nom::Err;

use crate::Subfield;

pub type ParseResult<'a, O> = Result<(&'a [u8], O), Err<()>>;

const US: char = '\x1F';

/// Parses a subfield name.
pub(crate) fn parse_subfield_name(i: &[u8]) -> ParseResult<char> {
    map(satisfy(|c| c.is_ascii_alphanumeric()), char::from)(i)
}

/// Parses a subfield value.
pub(crate) fn parse_subfield_value(i: &[u8]) -> ParseResult<&BStr> {
    recognize(many0(is_not("\x1E\x1F")))(i).map(|(i, o)| (i, o.as_bstr()))
}

/// Parses a subfield.
pub(crate) fn parse_subfield(i: &[u8]) -> ParseResult<Subfield> {
    map(
        preceded(
            char(US),
            cut(pair(parse_subfield_name, parse_subfield_value)),
        ),
        |(code, value)| Subfield { code, value },
    )(i)
}
