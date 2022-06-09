use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use bstr::{BStr, BString, ByteSlice};
use nom::character::complete::satisfy;
use nom::combinator::recognize;
use nom::sequence::tuple;
use nom::Finish;

use crate::{ParseError, ParseResult};

/// An immutable (read-only) PICA+ tag.
#[derive(Debug, PartialEq)]
pub struct TagRef<'a>(&'a BStr);

/// A mutable PICA+ tag.
#[derive(Clone, Debug, PartialEq)]
pub struct Tag(BString);

/// Parse a PICA+ tag
#[inline]
pub fn parse_tag<'a>(i: &'a [u8]) -> ParseResult<TagRef<'a>> {
    let (i, value) = recognize(tuple((
        satisfy(|c| c >= '0' && c <= '2'),
        satisfy(|c| c.is_ascii_digit()),
        satisfy(|c| c.is_ascii_digit()),
        satisfy(|c| c.is_ascii_uppercase() || c == '@'),
    )))(i)?;

    Ok((i, TagRef(value.as_bstr())))
}

impl<'a> TagRef<'a> {
    /// Creates an immutable PICA+ tag from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_core::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(TagRef::from_bytes(b"002@").is_ok());
    ///     assert!(TagRef::from_bytes(b"404A").is_err());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(match parse_tag(data).finish() {
            Ok((_, tag)) => tag,
            _ => return Err(ParseError::InvalidTag),
        })
    }
}

impl Tag {
    /// Creates an PICA+ tag from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_core::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(Tag::from_bytes(b"002@").is_ok());
    ///     assert!(Tag::from_bytes(b"404A").is_err());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        Ok(TagRef::from_bytes(data)?.into())
    }
}

impl From<TagRef<'_>> for Tag {
    fn from(tag: TagRef) -> Self {
        Tag(tag.0.into())
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

impl FromStr for Tag {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Tag::from_bytes(s.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TestResult;

    use nom_test_helpers::prelude::*;

    #[test]
    fn test_parse_tag() -> TestResult {
        assert_done_and_eq!(parse_tag(b"003@"), TagRef(b"003@".as_bstr()));
        assert_done_and_eq!(parse_tag(b"123@"), TagRef(b"123@".as_bstr()));
        assert_done_and_eq!(parse_tag(b"247C"), TagRef(b"247C".as_bstr()));

        assert_error!(parse_tag(b"456@"));
        assert_error!(parse_tag(b"0A2A"));
        assert_error!(parse_tag(b"01AA"));
        assert_error!(parse_tag(b"01Aa"));

        Ok(())
    }

    #[test]
    fn test_tag_ref_from_bytes() -> TestResult {
        assert_eq!(TagRef::from_bytes(b"003@")?, TagRef(b"003@".as_bstr()));
        assert!(TagRef::from_bytes(b"!003@").is_err());

        Ok(())
    }

    #[test]
    fn test_tag_from_tag_ref() -> TestResult {
        assert_eq!(
            Tag::from(TagRef::from_bytes(b"003@")?),
            Tag(BString::from("003@"))
        );

        Ok(())
    }
}
