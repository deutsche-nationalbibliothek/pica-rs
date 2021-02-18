//! This module provides a data structure and functions related to a PICA+
//! record.

use crate::select::{Outcome, Selector};

use nom::branch::alt;
use nom::bytes::streaming::{is_not, take_while_m_n};
use nom::character::complete::{
    char, multispace0, multispace1, none_of, one_of, satisfy,
};
use nom::combinator::{
    cut, map, map_opt, map_res, opt, recognize, success, value, verify,
};
use nom::error::{FromExternalError, ParseError};
use nom::multi::{count, fold_many0, many0, many1, many_m_n};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{combinator::all_consuming, Finish, IResult};

use serde::Serialize;
use std::borrow::Cow;
use std::cmp::PartialEq;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub enum ParsePicaError {
    InvalidSubfield,
    InvalidField,
    InvalidRecord,
    InvalidPath,
    InvalidFilter,
}

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1-6 hex
/// numerals. We will combine this later with parse_escaped_char to parse
/// sequences like \u{00AC}.
fn parse_unicode<'a, E>(i: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let parse_delimited_hex = preceded(
        char('u'),
        delimited(
            char('{'),
            take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit()),
            char('}'),
        ),
    );

    map_opt(
        map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16)),
        std::char::from_u32,
    )(i)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char<'a, E>(i: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        alt((
            parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(i)
}

/// Parse a non-empty block of text that doesn't include \ or ".
fn parse_literal<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    verify(is_not("\'\\"), |s: &str| !s.is_empty())(i)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a, E>(i: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, preceded(char('\\'), multispace1)),
    ))(i)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
pub(crate) fn parse_string<'a, E>(i: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    delimited(
        char('\''),
        fold_many0(parse_fragment, String::new(), |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        }),
        char('\''),
    )(i)
}

pub(crate) fn parse_subfield_name(i: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

pub(crate) fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_name, parse_subfield_value),
            |(name, value)| Subfield {
                name,
                value: value.into(),
            },
        ),
    )(i)
}

pub(crate) fn parse_field_tag(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        one_of("012"),
        count(one_of("0123456789"), 2),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
    )))(i)
}

pub(crate) fn parse_field_occurrence(i: &str) -> IResult<&str, Occurrence> {
    preceded(
        char('/'),
        map(
            recognize(many_m_n(2, 3, one_of("0123456789"))),
            Occurrence::new,
        ),
    )(i)
}

pub(crate) fn parse_field(i: &str) -> IResult<&str, Field> {
    terminated(
        map(
            pair(
                pair(parse_field_tag, opt(parse_field_occurrence)),
                preceded(char(' '), many0(parse_subfield)),
            ),
            |((tag, occurrence), subfields)| {
                Field::new(tag, occurrence, subfields)
            },
        ),
        char('\u{1e}'),
    )(i)
}

pub(crate) fn parse_record(i: &str) -> IResult<&str, Record> {
    map(many1(parse_field), Record::new)(i)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Subfield<'a> {
    pub(crate) name: char,
    pub(crate) value: Cow<'a, str>,
}

impl<'a> Subfield<'a> {
    /// Crates a new subfield
    ///
    /// # Arguments
    ///
    /// * `name` - An alpha-numeric ([0-9A-Za-z]) subfield code.
    /// * `value` - A string or string slice holding the subfield value.
    pub fn new<S>(name: char, value: S) -> Result<Self, ParsePicaError>
    where
        S: Into<Cow<'a, str>>,
    {
        let value = value.into();

        if !name.is_ascii_alphanumeric()
            || value.contains(&['\u{1e}', '\u{1f}'][..])
        {
            Err(ParsePicaError::InvalidSubfield)
        } else {
            Ok(Subfield { name, value })
        }
    }

    /// Decodes a subfield
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_subfield)(input).finish() {
            Ok((_, subfield)) => Ok(subfield),
            _ => Err(ParsePicaError::InvalidSubfield),
        }
    }

    /// Encodes a subfield
    pub fn encode(&self) -> String {
        format!("\u{1f}{}{}", self.name, self.value)
    }

    /// Returns the name of the subfield.
    pub fn name(&self) -> char {
        self.name
    }

    /// Returns the value of the subfield
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Occurrence<'a>(pub(crate) Cow<'a, str>);

impl<'a> Occurrence<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(value: S) -> Self {
        Self(value.into())
    }
}

impl<'a> Deref for Occurrence<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OccurrenceMatcher<'a> {
    Value(Cow<'a, str>),
    None,
    All,
}

impl<'a> OccurrenceMatcher<'a> {
    pub fn value<S: Into<Cow<'a, str>>>(value: S) -> Self {
        Self::Value(value.into())
    }

    pub fn all() -> Self {
        Self::All
    }

    pub fn none() -> Self {
        Self::None
    }
}

pub(crate) fn parse_occurrence_matcher(
    i: &str,
) -> IResult<&str, OccurrenceMatcher> {
    alt((
        preceded(
            char('/'),
            cut(alt((
                map(
                    recognize(many_m_n(2, 3, satisfy(|c| c.is_ascii_digit()))),
                    |value| OccurrenceMatcher::Value(Cow::Borrowed(value)),
                ),
                map(char('*'), |_| OccurrenceMatcher::All),
            ))),
        ),
        success(OccurrenceMatcher::None),
    ))(i)
}

impl<'a> PartialEq<Option<&Occurrence<'a>>> for OccurrenceMatcher<'a> {
    fn eq(&self, other: &Option<&Occurrence>) -> bool {
        match self {
            OccurrenceMatcher::All => true,
            OccurrenceMatcher::None => other.is_none(),
            OccurrenceMatcher::Value(lhs) => {
                if let Some(ref rhs) = other {
                    *lhs == rhs.0
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Field<'a> {
    pub(crate) tag: Cow<'a, str>,
    pub(crate) occurrence: Option<Occurrence<'a>>,
    pub(crate) subfields: Vec<Subfield<'a>>,
}

impl<'a> Field<'a> {
    /// Create a new field.
    ///
    /// # Example
    /// ```
    pub fn new<S>(
        tag: S,
        occurrence: Option<Occurrence<'a>>,
        subfields: Vec<Subfield<'a>>,
    ) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            tag: tag.into(),
            occurrence,
            subfields,
        }
    }

    /// Decodes a field
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice holding a PICA+ encoded field
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_field)(input).finish() {
            Ok((_, field)) => Ok(field),
            _ => Err(ParsePicaError::InvalidField),
        }
    }

    /// Returns the tag of the field.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the occurrence of the field.
    pub fn occurrence(&self) -> Option<&Occurrence> {
        self.occurrence.as_ref()
    }

    /// Returns the subfields of the field.
    pub fn subfields(&self) -> &Vec<Subfield> {
        &self.subfields
    }
}

impl<'a> Deref for Field<'a> {
    type Target = Vec<Subfield<'a>>;

    /// Dereferences the value
    fn deref(&self) -> &Vec<Subfield<'a>> {
        &self.subfields
    }
}

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct Record<'a>(Vec<Field<'a>>);

impl<'a> Record<'a> {
    /// Creates a new record
    ///
    /// # Arguments
    ///
    /// * A vector of [`Field`]s
    pub fn new(fields: Vec<Field<'a>>) -> Self {
        Self(fields)
    }

    /// Decodes a record
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice holding a PICA+ record
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_record)(input).finish() {
            Ok((_remaining, record)) => Ok(record),
            _ => Err(ParsePicaError::InvalidRecord),
        }
    }

    pub fn select(&self, selector: &Selector) -> Outcome {
        self.iter()
            .filter(|field| selector.tag == field.tag())
            .filter(|field| selector.occurrence == field.occurrence())
            .map(|field| field.subfields())
            .map(|subfields| {
                selector
                    .subfields
                    .iter()
                    .map(|name| {
                        subfields
                            .iter()
                            .filter(|subfield| subfield.name() == *name)
                            .map(|subfield| vec![subfield.value()])
                            .collect::<Vec<Vec<&str>>>()
                    })
                    .map(|x| {
                        if x.is_empty() {
                            Outcome::one()
                        } else {
                            Outcome(x)
                        }
                    })
                    .fold(Outcome::default(), |acc, x| acc * x)
            })
            .fold(Outcome::default(), |acc, x| acc + x)
    }
}

impl<'a> Deref for Record<'a> {
    type Target = Vec<Field<'a>>;

    fn deref(&self) -> &Vec<Field<'a>> {
        &self.0
    }
}
