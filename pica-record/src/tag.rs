use std::ops::Deref;

use bstr::{BStr, BString, ByteSlice};
use nom::character::complete::satisfy;
use nom::combinator::{all_consuming, map, recognize};
use nom::sequence::tuple;
use nom::Finish;

use crate::parser::ParseResult;
use crate::{OccurrenceRef, ParsePicaError};

/// A immutable PICA+ tag.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TagRef<'a>(pub(crate) &'a BStr);

impl<'a> TagRef<'a> {
    /// Create a new tag reference.
    ///
    /// # Panics
    ///
    /// This method panics if the tag is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let tag = TagRef::new("003@");
    ///     assert_eq!(tag, "003@");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(value: impl Into<&'a BStr>) -> Self {
        Self::from_bytes(value.into()).unwrap()
    }

    /// Creates an immutable PICA+ tag from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(TagRef::from_bytes(b"003@").is_ok());
    ///     assert!(TagRef::from_bytes(b"!03@").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParsePicaError> {
        all_consuming(parse_tag_ref)(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidTag)
            .map(|(_, tag)| tag)
    }

    /// Converts the immutable subfield into its mutable counterpart by
    /// consuming the source.
    pub fn into_owned(self) -> Tag {
        self.into()
    }

    /// Converts the immutable subfield into its mutable counterpart.
    pub fn to_owned(&self) -> Tag {
        self.clone().into()
    }
}

impl PartialEq<&str> for OccurrenceRef<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Parse a PICA+ tag (read-only).
pub fn parse_tag_ref(i: &[u8]) -> ParseResult<TagRef<'_>> {
    map(
        recognize(tuple((
            satisfy(|c| matches!(c, '0'..='2')),
            satisfy(|c| c.is_ascii_digit()),
            satisfy(|c| c.is_ascii_digit()),
            satisfy(|c| matches!(c, 'A'..='Z' | '@')),
        ))),
        |value| TagRef(ByteSlice::as_bstr(value)),
    )(i)
}

impl PartialEq<&str> for TagRef<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl Deref for TagRef<'_> {
    type Target = BStr;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// A mutable PICA+ tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag(pub(crate) BString);

impl Tag {
    /// Create a new tag.
    ///
    /// # Panics
    ///
    /// This method panics if the tag is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let tag = Tag::new("003@");
    ///     assert_eq!(tag, "003@");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(value: impl Into<BString>) -> Self {
        Self::from_bytes(&value.into()).unwrap()
    }

    /// Creates an immutable PICA+ tag from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(Tag::from_bytes(b"003@").is_ok());
    ///     assert!(Tag::from_bytes(b"003!").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParsePicaError> {
        Ok(TagRef::from_bytes(data)?.into())
    }
}

impl Deref for Tag {
    type Target = BString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<TagRef<'_>> for Tag {
    #[inline]
    fn from(tag: TagRef<'_>) -> Self {
        Self(tag.0.into())
    }
}

impl PartialEq<&str> for Tag {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_tag_ref_new() {
        let tag = TagRef::new("003@");
        assert_eq!(tag, TagRef("003@".into()));
        assert_eq!(tag, "003@")
    }

    #[test]
    #[should_panic(expected = "InvalidTag")]
    fn test_tag_ref_invalid() {
        TagRef::new("003!");
    }

    #[test]
    fn test_tag_new() {
        let tag = Tag::new("003@");
        assert_eq!(tag, Tag("003@".into()));
        assert_eq!(tag, "003@")
    }

    #[test]
    #[should_panic(expected = "InvalidTag")]
    fn test_tag_invalid() {
        Tag::new("003!");
    }

    #[test]
    fn test_parse_tag_ref() {
        for tag in ["003@", "002@", "123@", "247C"] {
            assert_done_and_eq!(
                parse_tag_ref(tag.as_bytes()),
                TagRef::new(tag)
            )
        }

        for tag in ["456@", "0A2A", "01AA", "01Aa"] {
            assert_error!(parse_tag_ref(tag.as_bytes()))
        }
    }

    #[quickcheck]
    fn test_parse_arbitrary_tag(tag: Tag) -> bool {
        TagRef::from_bytes(tag.to_string().as_bytes()).is_ok()
    }
}
