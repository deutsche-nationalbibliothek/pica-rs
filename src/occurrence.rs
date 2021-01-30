use nom::branch::alt;
use nom::character::complete::{char, satisfy};
use nom::combinator::{cut, map, recognize, success};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_occurrence_matcher() {
        assert_eq!(
            parse_occurrence_matcher("abc"),
            Ok(("abc", OccurrenceMatcher::None))
        );

        assert_eq!(
            parse_occurrence_matcher("/01"),
            Ok(("", OccurrenceMatcher::value("01")))
        );

        assert_eq!(
            parse_occurrence_matcher("/*"),
            Ok(("", OccurrenceMatcher::all()))
        );
    }

    #[test]
    fn test_partial_eq() {
        assert_ne!(OccurrenceMatcher::none(), Some(&Occurrence::new("01")));
        assert_eq!(OccurrenceMatcher::none(), None);

        assert_eq!(OccurrenceMatcher::all(), Some(&Occurrence::new("01")));
        assert_eq!(OccurrenceMatcher::all(), None);

        assert_ne!(OccurrenceMatcher::value("01"), None);
        assert_eq!(
            OccurrenceMatcher::value("01"),
            Some(&Occurrence::new("01"))
        );
        assert_ne!(
            OccurrenceMatcher::value("01"),
            Some(&Occurrence::new("02"))
        );
    }
}
