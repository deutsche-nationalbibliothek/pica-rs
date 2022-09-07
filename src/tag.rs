//! This module contains data structures and functions related to
//! PICA+ tags.

use std::fmt;
use std::ops::Deref;

use bstr::BString;
use nom::branch::alt;
use nom::character::complete::{char, one_of, satisfy};
use nom::combinator::{all_consuming, map, recognize};
use nom::multi::count;
use nom::sequence::tuple;
use nom::Finish;

use crate::common::ParseResult;
use crate::error::Error;

/// A PICA+ tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag(pub(crate) BString);

#[derive(Debug, PartialEq, Eq)]
pub enum Level {
    Main,
    Local,
    Copy,
}

/// Parses a PICA+ tag.
///
/// ```ebnf
/// <tag> ::= [012] [0-9]{2} ([A-Z] | '@')
/// ```
#[inline]
pub(crate) fn parse_tag(i: &[u8]) -> ParseResult<Tag> {
    map(
        recognize(tuple((
            one_of("012"),
            count(satisfy(|c| c.is_ascii_digit()), 2),
            alt((satisfy(|c| c.is_ascii_uppercase()), char('@'))),
        ))),
        Tag::from_unchecked,
    )(i)
}

impl Tag {
    /// Creates a PICA+ tag from a string slice.
    ///
    /// If an invalid tag is given, an error is returned.
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
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        let data = data.as_ref();

        match all_consuming(parse_tag)(data.as_bytes()).finish() {
            Ok((_, tag)) => Ok(tag),
            Err(_) => Err(Error::InvalidTag("Invalid tag".to_string())),
        }
    }

    /// Creates a new `Tag` without checking the input
    pub(crate) fn from_unchecked<S: Into<BString>>(tag: S) -> Self {
        Self(tag.into())
    }

    /// Returns the `Level` of the tag.
    /// # Example
    ///
    /// ```rust
    /// use pica::{Level, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert_eq!(Tag::new("003@")?.level(), Level::Main);
    ///     Ok(())
    /// }
    /// ```
    pub fn level(&self) -> Level {
        match self.0.first() {
            Some(b'0') => Level::Main,
            Some(b'1') => Level::Local,
            Some(b'2') => Level::Copy,
            Some(_) | None => {
                panic!("Expected tag to start with '0', '1' or '2'.");
            }
        }
    }
}

impl Deref for Tag {
    type Target = BString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Tag {
    /// Format the tag in a human-readable format.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<&str> for Tag {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Tag {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let p1 = *g.choose(b"012").unwrap();
        let p2 = *g.choose(b"0123456789").unwrap();
        let p3 = *g.choose(b"0123456789").unwrap();
        let p4 = *g.choose(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ@").unwrap();

        Tag::from_unchecked(vec![p1, p2, p3, p4])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_tag() -> TestResult {
        assert_eq!(parse_tag(b"003@").unwrap().1, Tag::new("003@")?);
        assert_eq!(parse_tag(b"123@").unwrap().1, Tag::new("123@")?);
        assert_eq!(parse_tag(b"247C").unwrap().1, Tag::new("247C")?);
        assert!(parse_tag(b"456@").is_err());
        assert!(parse_tag(b"0A2A").is_err());
        assert!(parse_tag(b"01AA").is_err());
        assert!(parse_tag(b"01Aa").is_err());
        Ok(())
    }

    #[test]
    fn test_tag_new() -> TestResult {
        assert_eq!(Tag::new("003@")?, Tag(BString::from("003@")));
        assert!(Tag::new("003@ ").is_err());
        Ok(())
    }

    #[test]
    fn test_tag_from_unchecked() -> TestResult {
        assert_eq!(
            Tag::from_unchecked("003@"),
            Tag(BString::from("003@"))
        );
        Ok(())
    }

    #[test]
    fn test_tag_to_string() -> TestResult {
        assert_eq!(Tag::from_unchecked("003@").to_string(), "003@");
        Ok(())
    }

    #[test]
    fn test_tag_level() -> TestResult {
        assert_eq!(Tag::new("003@")?.level(), Level::Main);
        assert_eq!(Tag::new("123A")?.level(), Level::Local);
        assert_eq!(Tag::new("247C")?.level(), Level::Copy);
        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "Expected tag to start with '0', '1' or '2'."
    )]
    fn test_invalid_tag_level() {
        Tag::from_unchecked("345A").level();
    }

    #[quickcheck]
    fn tag_from_string_representation_is_identity(tag: Tag) -> bool {
        tag == Tag::new(tag.to_string()).unwrap()
    }
}
