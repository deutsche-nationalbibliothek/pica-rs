use std::fmt;
use std::ops::RangeFrom;

use nom::branch::alt;
use nom::character::complete::{char, one_of};
use nom::combinator::{all_consuming, cut, map, value};
use nom::multi::many1;
use nom::sequence::{preceded, terminated, tuple};
use nom::{AsChar, FindToken, Finish, IResult, InputIter, InputLength, Slice};

use pica_core::parser::parse_tag;
use pica_core::{ParseResult, Tag};

use crate::ParseError;

#[derive(Debug, PartialEq, Eq)]
pub enum TagMatcher {
    Some(Tag),
    Pattern(Vec<char>, Vec<char>, Vec<char>, Vec<char>),
}

impl fmt::Display for TagMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Some(tag) => write!(f, "{}", tag),
            Self::Pattern(ref p1, ref p2, ref p3, ref p4) => {
                let fmt_p = |p: &Vec<char>| -> String {
                    if p.len() > 1 {
                        format!("[{}]", String::from_iter(p))
                    } else {
                        String::from_iter(p)
                    }
                };

                write!(
                    f,
                    "{}{}{}{}",
                    fmt_p(p1),
                    fmt_p(p2),
                    fmt_p(p3),
                    fmt_p(p4)
                )
            }
        }
    }
}

impl TagMatcher {
    /// Creates a tag matcher from a string slice.
    ///
    /// If an invalid tag matcher is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::TagMatcher;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(TagMatcher::new("003@").is_ok());
    ///     assert!(TagMatcher::new("0[12]3A").is_ok());
    ///     assert!(TagMatcher::new("023!").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, ParseError> {
        let data = data.as_ref();

        match all_consuming(parse_tag_matcher)(data.as_bytes()).finish() {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(ParseError::InvalidTagMatcher),
        }
    }

    /// Returns true, if and only if the given tag matches against the
    /// tag matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::Tag;
    /// use pica_matcher::TagMatcher;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = TagMatcher::new("012[A@]")?;
    ///     assert!(matcher.is_match(&Tag::from_str("012A")?));
    ///     assert!(matcher.is_match(&Tag::from_str("012@")?));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, tag: &Tag) -> bool {
        match self {
            TagMatcher::Some(lhs) => lhs == tag,
            TagMatcher::Pattern(p0, p1, p2, p3) => {
                p0.contains(&(tag[0] as char))
                    && p1.contains(&(tag[1] as char))
                    && p2.contains(&(tag[2] as char))
                    && p3.contains(&(tag[3] as char))
            }
        }
    }
}

impl From<Tag> for TagMatcher {
    fn from(tag: Tag) -> Self {
        Self::Some(tag)
    }
}

fn parse_character_class<I, T, E: nom::error::ParseError<I>>(
    list: T,
) -> impl FnMut(I) -> IResult<I, Vec<char>, E>
where
    I: Slice<RangeFrom<usize>> + InputIter + Clone + InputLength,
    <I as InputIter>::Item: AsChar + Copy,
    T: FindToken<<I as InputIter>::Item> + Clone,
{
    alt((
        preceded(
            char('['),
            cut(terminated(many1(one_of(list.clone())), char(']'))),
        ),
        map(one_of(list), |x| vec![x]),
    ))
}

pub fn parse_tag_matcher(i: &[u8]) -> ParseResult<TagMatcher> {
    alt((
        map(parse_tag, |tag| TagMatcher::Some(Tag::from(tag))),
        map(
            tuple((
                alt((
                    value("012".chars().collect(), char('.')),
                    parse_character_class("012"),
                )),
                alt((
                    value("0123456789".chars().collect(), char('.')),
                    parse_character_class("0123456789"),
                )),
                alt((
                    value("0123456789".chars().collect(), char('.')),
                    parse_character_class("0123456789"),
                )),
                alt((
                    value(
                        "ABCDEFGHIJKLMNOPQRSTUVWXYZ@".chars().collect(),
                        char('.'),
                    ),
                    parse_character_class("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
                )),
            )),
            |(p1, p2, p3, p4)| TagMatcher::Pattern(p1, p2, p3, p4),
        ),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestResult;
    use std::str::FromStr;

    #[test]
    fn test_tag_matcher() -> TestResult {
        let matcher = TagMatcher::new("012A")?;
        assert!(matcher.is_match(&Tag::from_str("012A")?));
        assert!(!matcher.is_match(&Tag::from_str("012@")?));

        let matcher = TagMatcher::new("[01][34][56][AB]")?;
        assert!(matcher.is_match(&Tag::from_str("035A")?));
        assert!(matcher.is_match(&Tag::from_str("146B")?));

        let matcher = TagMatcher::new(".12A")?;
        assert!(matcher.is_match(&Tag::from_str("012A")?));
        assert!(matcher.is_match(&Tag::from_str("112A")?));
        assert!(matcher.is_match(&Tag::from_str("212A")?));

        let matcher = TagMatcher::new("0.2A")?;
        assert!(matcher.is_match(&Tag::from_str("002A")?));
        assert!(matcher.is_match(&Tag::from_str("012A")?));
        assert!(matcher.is_match(&Tag::from_str("022A")?));
        assert!(matcher.is_match(&Tag::from_str("032A")?));
        assert!(matcher.is_match(&Tag::from_str("042A")?));
        assert!(matcher.is_match(&Tag::from_str("052A")?));
        assert!(matcher.is_match(&Tag::from_str("062A")?));
        assert!(matcher.is_match(&Tag::from_str("072A")?));
        assert!(matcher.is_match(&Tag::from_str("082A")?));
        assert!(matcher.is_match(&Tag::from_str("092A")?));

        let matcher = TagMatcher::new("01.A")?;
        assert!(matcher.is_match(&Tag::from_str("010A")?));
        assert!(matcher.is_match(&Tag::from_str("011A")?));
        assert!(matcher.is_match(&Tag::from_str("012A")?));
        assert!(matcher.is_match(&Tag::from_str("013A")?));
        assert!(matcher.is_match(&Tag::from_str("014A")?));
        assert!(matcher.is_match(&Tag::from_str("015A")?));
        assert!(matcher.is_match(&Tag::from_str("016A")?));
        assert!(matcher.is_match(&Tag::from_str("017A")?));
        assert!(matcher.is_match(&Tag::from_str("018A")?));
        assert!(matcher.is_match(&Tag::from_str("019A")?));

        let matcher = TagMatcher::new("012.")?;
        assert!(matcher.is_match(&Tag::from_str("012A")?));
        assert!(matcher.is_match(&Tag::from_str("012B")?));
        assert!(matcher.is_match(&Tag::from_str("012C")?));
        assert!(matcher.is_match(&Tag::from_str("012@")?));

        let matcher = TagMatcher::new("0...")?;
        assert!(matcher.is_match(&Tag::from_str("012A")?));
        assert!(matcher.is_match(&Tag::from_str("023B")?));

        assert!(TagMatcher::new("412A").is_err());
        assert!(TagMatcher::new("0A2A").is_err());
        assert!(TagMatcher::new("01AA").is_err());
        assert!(TagMatcher::new("0123").is_err());

        Ok(())
    }

    #[test]
    fn test_tag_matcher_to_string() -> TestResult {
        assert_eq!(TagMatcher::new("012A")?.to_string(), "012A");
        assert_eq!(TagMatcher::new("[01]12A")?.to_string(), "[01]12A");
        assert_eq!(TagMatcher::new("[0]12A")?.to_string(), "012A");
        Ok(())
    }
}
