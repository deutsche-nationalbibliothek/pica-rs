use std::borrow::Cow;

use nom::branch::alt;
use nom::character::complete::{char, satisfy};
use nom::combinator::{cut, map, recognize};
use nom::multi::many_m_n;
use nom::sequence::preceded;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum OccurrenceMatcher<'a> {
    Value(Cow<'a, str>),
    None,
    All,
}

impl<'a> OccurrenceMatcher<'a> {
    pub(crate) fn equals(&self, value: &Option<&str>) -> bool {
        match self {
            OccurrenceMatcher::All => true,
            OccurrenceMatcher::None => value.is_none(),
            OccurrenceMatcher::Value(val1) => {
                if let Some(ref val2) = value {
                    val1 == val2
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
