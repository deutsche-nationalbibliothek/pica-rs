use std::borrow::Cow;

use nom::branch::alt;
use nom::character::complete::{char, satisfy};
use nom::combinator::{cut, map, recognize};
use nom::multi::many_m_n;
use nom::sequence::preceded;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum Occurrence<'a> {
    Value(Cow<'a, str>),
    None,
    All,
}

impl<'a> Occurrence<'a> {
    pub(crate) fn equals(&self, value: &Option<Cow<'a, str>>) -> bool {
        match self {
            Occurrence::All => true,
            Occurrence::None => value.is_none(),
            Occurrence::Value(val1) => {
                if let Some(val2) = value {
                    val1 == val2
                } else {
                    false
                }
            }
        }
    }
}

pub(crate) fn parse_occurrence(i: &str) -> IResult<&str, Occurrence> {
    preceded(
        char('/'),
        cut(alt((
            map(
                recognize(many_m_n(2, 3, satisfy(|c| c.is_ascii_digit()))),
                |value| Occurrence::Value(Cow::Borrowed(value)),
            ),
            map(char('*'), |_| Occurrence::All),
        ))),
    )(i)
}
