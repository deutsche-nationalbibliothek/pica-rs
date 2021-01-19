use nom::branch::alt;
use nom::character::complete::{char, satisfy};
use nom::combinator::{cut, map, recognize};
use nom::multi::many_m_n;
use nom::sequence::preceded;
use nom::IResult;

use serde::Serialize;
use std::borrow::Cow;
use std::cmp::PartialEq;
use std::ops::Deref;

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

#[derive(Debug, PartialEq)]
pub enum OccurrenceMatcher<'a> {
    Value(Cow<'a, str>),
    None,
    All,
}

impl<'a> OccurrenceMatcher<'a> {
    pub(crate) fn equals(&self, value: Option<&Occurrence>) -> bool {
        match self {
            OccurrenceMatcher::All => true,
            OccurrenceMatcher::None => value.is_none(),
            OccurrenceMatcher::Value(val1) => {
                if let Some(val2) = value {
                    val1 == &val2.0
                } else {
                    false
                }
            }
        }
    }
}

pub(crate) fn parse_occurrence_matcher(
    i: &str,
) -> IResult<&str, OccurrenceMatcher> {
    preceded(
        char('/'),
        cut(alt((
            map(
                recognize(many_m_n(2, 3, satisfy(|c| c.is_ascii_digit()))),
                |value| OccurrenceMatcher::Value(Cow::Borrowed(value)),
            ),
            map(char('*'), |_| OccurrenceMatcher::All),
        ))),
    )(i)
}
