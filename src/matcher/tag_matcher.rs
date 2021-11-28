use std::ops::RangeFrom;

use nom::branch::alt;
use nom::character::complete::{char, one_of};
use nom::combinator::{all_consuming, cut, map};
use nom::error::ParseError;
use nom::multi::many1;
use nom::sequence::{preceded, terminated, tuple};
use nom::{AsChar, FindToken, Finish, IResult, InputIter, InputLength, Slice};

use crate::common::ParseResult;
use crate::tag::{parse_tag, Tag};
use crate::Error;

#[derive(Debug, PartialEq)]
pub enum TagMatcher {
    Some(Tag),
    Pattern(Vec<char>, Vec<char>, Vec<char>, Vec<char>),
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
                parse_character_class("012"),
                parse_character_class("0123456789"),
                parse_character_class("0123456789"),
                parse_character_class("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
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

        assert!(TagMatcher::new("412A").is_err());
        assert!(TagMatcher::new("0A2A").is_err());
        assert!(TagMatcher::new("01AA").is_err());
        assert!(TagMatcher::new("0123").is_err());

        Ok(())
    }

    #[quickcheck]
    fn tag_matcher_quickcheck(tag: Tag) -> bool {
        TagMatcher::from(tag.clone()).is_match(&tag)
    }
}
