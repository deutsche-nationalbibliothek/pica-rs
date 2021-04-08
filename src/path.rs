//! Pica+ Path
//!
//! A path is a query syntax to address values within a pica+ record. The path
//! consists of a [`Field`] tag and a [`Subfield`] name. A [`Field`] occurrence
//! or an index is optional
//!
//! # Grammar
//!
//! ```text
//! path       ::= tag occurrence? name
//! tag        ::= [012] [0-9]{2} ([A-Z] | '@')
//! occurrence ::= '/' [0-9]{2,3}
//! name       ::= [a-z] | [A-Z] | [0-9]
//! ```

use nom::character::complete::{char, multispace0};
use nom::combinator::cut;
use nom::sequence::{preceded, terminated, tuple};

use crate::record::{
    parse_field_occurrence, parse_field_tag, parse_subfield_code, ParseResult,
};
use crate::Occurrence;

use bstr::BString;

#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    pub(crate) tag: BString,
    pub(crate) occurrence: Occurrence,
    pub(crate) code: char,
}

impl Path {
    /// Parse a path from a byte slice.
    #[allow(clippy::result_unit_err)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, ()> {
        parse_path(data).map(|(_, path)| path).map_err(|_| ())
    }
}

pub fn parse_path(i: &[u8]) -> ParseResult<Path> {
    let (i, (tag, occurrence, code)) = tuple((
        preceded(multispace0, parse_field_tag),
        parse_field_occurrence,
        preceded(char('.'), cut(terminated(parse_subfield_code, multispace0))),
    ))(i)?;

    Ok((
        i,
        Path {
            tag,
            occurrence,
            code,
        },
    ))
}
