//! Pica+ Path
//!
//! A path is a query syntax to address values within a pica+ record. The path
//! consists of a [`Field`] tag and a [`Subfield`] name. A [`Field`] occurrence
//! or an index is optional
//!
//! # Grammar
//!
//! ```text
//! path       ::= tag occurrence? name index?
//! tag        ::= [012] [0-9]{2} ([A-Z] | '@')
//! occurrence ::= '/' [0-9]{2,3}
//! name       ::= [a-z] | [A-Z] | [0-9]
//! index      ::= '[' [0-9]+ ']'
//! ```

use crate::error::ParsePicaError;
use crate::legacy::Occurrence;
use crate::parser::{
    parse_field_occurrence, parse_field_tag, parse_subfield_name,
};

use nom::character::complete::{char, digit1, multispace0};
use nom::combinator::{cut, map, opt};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Path<'a> {
    tag: String,
    occurrence: Option<Occurrence<'a>>,
    name: char,
    index: Option<usize>,
}

impl<'a> Path<'a> {
    pub fn new<S>(
        tag: S,
        occurrence: Option<Occurrence<'a>>,
        name: char,
        index: Option<usize>,
    ) -> Self
    where
        S: Into<String>,
    {
        Path {
            tag: tag.into(),
            occurrence,
            name,
            index,
        }
    }

    pub fn decode(s: &'a str) -> Result<Self, ParsePicaError> {
        match parse_path(s) {
            Ok((_, path)) => Ok(path),
            _ => Err(ParsePicaError::InvalidPath),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn occurrence(&self) -> Option<&Occurrence> {
        self.occurrence.as_ref()
    }

    pub fn name(&self) -> char {
        self.name
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }
}

impl<'a> fmt::Display for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)?;

        if let Some(occurrence) = self.occurrence() {
            write!(f, "/{}", occurrence.0)?;
        };

        write!(f, ".{}", self.name)?;

        if let Some(index) = self.index {
            write!(f, "[{}]", index)?;
        }

        Ok(())
    }
}

fn parse_index(i: &str) -> IResult<&str, usize> {
    preceded(
        char('['),
        cut(terminated(
            map(digit1, |v: &str| v.parse::<usize>().unwrap()),
            char(']'),
        )),
    )(i)
}

pub fn parse_path(i: &str) -> IResult<&str, Path> {
    map(
        tuple((
            preceded(multispace0, parse_field_tag),
            opt(parse_field_occurrence),
            preceded(char('.'), parse_subfield_name),
            opt(parse_index),
            multispace0,
        )),
        |(tag, occurrence, name, index, _)| {
            Path::new(tag, occurrence, name, index)
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path("012A/000.a[0]"),
            Ok((
                "",
                Path::new("012A", Some(Occurrence::new("000")), 'a', Some(0))
            ))
        );

        assert!(parse_path("012A/000.a[a]").is_err());
    }
}
