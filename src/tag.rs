//! This module contains data structures and functions related to
//! PICA+ tags.

use crate::parser::{parse_character_class, ParseResult};

use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::ops::Deref;

use nom::branch::alt;
use nom::character::complete::one_of;
use nom::combinator::{map, recognize};
use nom::sequence::tuple;

use bstr::BString;

#[derive(Debug, PartialEq, Eq)]
pub struct Tag(pub(crate) BString);

#[derive(Debug)]
pub struct ParseTagError(pub(crate) String);

impl std::error::Error for ParseTagError {}

impl fmt::Display for ParseTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for Tag {
    type Target = BString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<&str> for Tag {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<BString> for Tag {
    #[inline]
    fn eq(&self, other: &BString) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Tag> for &str {
    #[inline]
    fn eq(&self, other: &Tag) -> bool {
        *self == other.0
    }
}

impl fmt::Display for Tag {
    /// Format the field in a human-readable format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, Field, Occurrence, Subfield, Tag};
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("012A")?,
    ///         Some(Occurrence::new("01")?),
    ///         vec![
    ///             Subfield::new('0', "123456789X")?,
    ///             Subfield::new('a', "foo")?,
    ///         ],
    ///     )?;
    ///
    ///     assert_eq!(format!("{}", field), "012A/01 $0123456789X$afoo");
    ///
    ///     Ok(())
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Tag {
    /// Creates a PICA+ tag from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(Tag::new("003@").is_ok());
    ///     assert!(Tag::new("303@").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new(s: &str) -> Result<Self, ParseTagError> {
        Self::from_bytes(s)
    }

    /// Parses a PICA+ tag.
    ///
    /// # Grammar
    ///
    /// ```ebnf
    /// tag ::= [0-2] [0-9]{2} ([a-z] | [A-Z] | "@")
    /// ```
    #[inline]
    pub(crate) fn parse_tag(i: &[u8]) -> ParseResult<Tag> {
        map(
            recognize(tuple((
                one_of("012"),
                one_of("0123456789"),
                one_of("0123456789"),
                one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
            ))),
            Tag::from_unchecked,
        )(i)
    }

    /// Creates a new `Tag` without checking the input.
    #[inline]
    pub(crate) fn from_unchecked<S: Into<BString>>(tag: S) -> Self {
        Self(tag.into())
    }

    /// Creates a new `Tag` from a byte vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert_eq!(Tag::from_bytes("003@").unwrap(), "003@");
    ///     assert!(Tag::from_bytes("303@").is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes<T>(data: T) -> Result<Self, ParseTagError>
    where
        T: Into<Vec<u8>>,
    {
        Self::parse_tag(&data.into())
            .map_err(|_| ParseTagError("invalid tag".to_string()))
            .map(|(_, tag)| tag)
    }

    /// Returns the level of the tag.
    ///
    /// This function assumes a valid tag; invalid tags created by
    /// [`Self::from_unchecked`] may cause a panic.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Tag;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tag = Tag::new("003@")?;
    ///     assert_eq!(tag.level(), 0);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn level(&self) -> u8 {
        match self.0.get(0).unwrap() {
            b'0' => 0,
            b'1' => 1,
            b'2' => 2,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TagMatcher {
    Some(Tag),
    Pattern(Vec<char>, Vec<char>, Vec<char>, Vec<char>),
}

#[derive(Debug)]
pub struct ParseTagMatcherError(pub(crate) String);

impl std::error::Error for ParseTagMatcherError {}

impl fmt::Display for ParseTagMatcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TagMatcher {
    /// Creates a tag matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::TagMatcher;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(TagMatcher::new("0[12]3[A@]").is_ok());
    ///     assert!(TagMatcher::new("3[12]3@").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new(s: &str) -> Result<Self, ParseTagMatcherError> {
        Self::from_bytes(s)
    }

    #[inline]
    pub(crate) fn parse_tag_matcher(i: &[u8]) -> ParseResult<Self> {
        alt((
            map(Tag::parse_tag, TagMatcher::Some),
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

    /// Creates a new `TagMatcher` from a byte vector.
    pub(crate) fn from_bytes<T>(data: T) -> Result<Self, ParseTagMatcherError>
    where
        T: Into<Vec<u8>>,
    {
        Self::parse_tag_matcher(&data.into())
            .map_err(|_| {
                ParseTagMatcherError("invalid tag matcher".to_string())
            })
            .map(|(_, matcher)| matcher)
    }
}

impl PartialEq<TagMatcher> for Tag {
    fn eq(&self, other: &TagMatcher) -> bool {
        match other {
            TagMatcher::Some(tag) => self == tag,
            TagMatcher::Pattern(p0, p1, p2, p3) => {
                p0.contains(&(self[0] as char))
                    && p1.contains(&(self[1] as char))
                    && p2.contains(&(self[2] as char))
                    && p3.contains(&(self[3] as char))
            }
        }
    }
}

impl PartialEq<Tag> for TagMatcher {
    fn eq(&self, other: &Tag) -> bool {
        *other == *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_matcher_from_str() {
        assert_eq!(
            TagMatcher::new("003@").unwrap(),
            TagMatcher::Some(Tag::new("003@").unwrap())
        );
        assert_eq!(
            TagMatcher::new("[01][234][56][A@]").unwrap(),
            TagMatcher::Pattern(
                vec!['0', '1'],
                vec!['2', '3', '4'],
                vec!['5', '6'],
                vec!['A', '@']
            ),
        );
        assert!(TagMatcher::new("300A").is_err());
    }

    #[test]
    fn test_tag_level() {
        assert_eq!(Tag::new("003@").unwrap().level(), 0);
        assert_eq!(Tag::new("123A").unwrap().level(), 1);
        assert_eq!(Tag::new("234A").unwrap().level(), 2);
    }

    #[test]
    fn test_tag_from_unchecked() {
        assert_eq!(Tag::from_unchecked("003@"), Tag(BString::from("003@")));
        assert_eq!(Tag::from_unchecked("300A"), Tag(BString::from("300A")));
    }

    #[test]
    fn test_tag_from_bytes() {
        assert_eq!(
            Tag::from_bytes("003@").unwrap(),
            Tag(BString::from("003@"))
        );
        assert!(Tag::from_bytes("300A").is_err());
    }

    #[test]
    fn test_partial_eq() {
        assert_eq!(Tag::new("003@").unwrap(), "003@");
        assert_eq!("003@", Tag::new("003@").unwrap());

        let tag = Tag::new("012A").unwrap();
        let matcher = TagMatcher::new("0[12][32]A").unwrap();
        assert_eq!(tag, matcher);
        assert_eq!(matcher, tag);
    }
}
