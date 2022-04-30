use std::fmt;
use std::ops::RangeFrom;

use nom::branch::alt;
use nom::character::complete::{char, one_of};
use nom::combinator::{all_consuming, cut, map, value};
use nom::error::ParseError;
use nom::multi::many1;
use nom::sequence::{preceded, terminated, tuple};
use nom::{AsChar, FindToken, Finish, IResult, InputIter, InputLength, Slice};

use pica_core::ParseResult;

use crate::tag::{parse_tag, Tag};
use crate::Error;

#[derive(Debug, PartialEq)]
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
    /// use pica::matcher::TagMatcher;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(TagMatcher::new("003@").is_ok());
    ///     assert!(TagMatcher::new("0[12]3A").is_ok());
    ///     assert!(TagMatcher::new("023!").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        let data = data.as_ref();

        match all_consuming(parse_tag_matcher)(data.as_bytes()).finish() {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(Error::InvalidMatcher(format!(
                "Expected valid tag matcher, got '{}'",
                data
            ))),
        }
    }

    /// Returns true, if and only if the given tag matches against the
    /// tag matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::TagMatcher;
    /// use pica::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = TagMatcher::new("012[A@]")?;
    ///     assert!(matcher.is_match(&Tag::new("012A")?));
    ///     assert!(matcher.is_match(&Tag::new("012@")?));
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

fn parse_character_class<I, T, E: ParseError<I>>(
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

pub(crate) fn parse_tag_matcher(i: &[u8]) -> ParseResult<TagMatcher> {
    alt((
        map(parse_tag, TagMatcher::Some),
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
    use crate::test::TestResult;

    #[test]
    fn test_tag_matcher() -> TestResult {
        let matcher = TagMatcher::new("012A")?;
        assert!(matcher.is_match(&Tag::new("012A")?));
        assert!(!matcher.is_match(&Tag::new("012@")?));

        let matcher = TagMatcher::new("[01][34][56][AB]")?;
        assert!(matcher.is_match(&Tag::new("035A")?));
        assert!(matcher.is_match(&Tag::new("146B")?));

        let matcher = TagMatcher::new(".12A")?;
        assert!(matcher.is_match(&Tag::new("012A")?));
        assert!(matcher.is_match(&Tag::new("112A")?));
        assert!(matcher.is_match(&Tag::new("212A")?));

        let matcher = TagMatcher::new("0.2A")?;
        assert!(matcher.is_match(&Tag::new("002A")?));
        assert!(matcher.is_match(&Tag::new("012A")?));
        assert!(matcher.is_match(&Tag::new("022A")?));
        assert!(matcher.is_match(&Tag::new("032A")?));
        assert!(matcher.is_match(&Tag::new("042A")?));
        assert!(matcher.is_match(&Tag::new("052A")?));
        assert!(matcher.is_match(&Tag::new("062A")?));
        assert!(matcher.is_match(&Tag::new("072A")?));
        assert!(matcher.is_match(&Tag::new("082A")?));
        assert!(matcher.is_match(&Tag::new("092A")?));

        let matcher = TagMatcher::new("01.A")?;
        assert!(matcher.is_match(&Tag::new("010A")?));
        assert!(matcher.is_match(&Tag::new("011A")?));
        assert!(matcher.is_match(&Tag::new("012A")?));
        assert!(matcher.is_match(&Tag::new("013A")?));
        assert!(matcher.is_match(&Tag::new("014A")?));
        assert!(matcher.is_match(&Tag::new("015A")?));
        assert!(matcher.is_match(&Tag::new("016A")?));
        assert!(matcher.is_match(&Tag::new("017A")?));
        assert!(matcher.is_match(&Tag::new("018A")?));
        assert!(matcher.is_match(&Tag::new("019A")?));

        let matcher = TagMatcher::new("012.")?;
        assert!(matcher.is_match(&Tag::new("012A")?));
        assert!(matcher.is_match(&Tag::new("012B")?));
        assert!(matcher.is_match(&Tag::new("012C")?));
        assert!(matcher.is_match(&Tag::new("012@")?));

        let matcher = TagMatcher::new("0...")?;
        assert!(matcher.is_match(&Tag::new("012A")?));
        assert!(matcher.is_match(&Tag::new("023B")?));

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

    #[quickcheck]
    fn tag_matcher_quickcheck(tag: Tag) -> bool {
        TagMatcher::from(tag.clone()).is_match(&tag)
    }
}
