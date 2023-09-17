use std::fmt::{self, Display};
use std::ops::{Deref, Index};
use std::str;

use bstr::{BStr, BString, ByteSlice};
use nom::character::complete::satisfy;
use nom::combinator::{all_consuming, map, recognize};
use nom::sequence::tuple;
use nom::Finish;

use crate::parser::ParseResult;
use crate::{Level, ParsePicaError};

/// A PICA+ tag.
#[derive(Eq, Debug, Clone)]
pub struct Tag<T: AsRef<[u8]>>(T);

/// A immutable PICA+ tag.
pub type TagRef<'a> = Tag<&'a BStr>;

/// A mutable PICA+ tag.
pub type TagMut = Tag<BString>;

impl<'a, T: AsRef<[u8]> + From<&'a BStr> + Display> Tag<T> {
    /// Create a new PICA+ tag.
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
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(value: impl Into<T>) -> Self {
        let value = value.into();

        all_consuming(parse_tag)(value.as_ref())
            .map_err(|_| ParsePicaError::InvalidTag)
            .unwrap();

        Self(value)
    }

    /// Creates an PICA+ tag from a byte slice.
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
        all_consuming(parse_tag)(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidTag)
            .map(|(_, tag)| Tag(tag.into()))
    }

    /// Creates a new Tag without checking the input.
    pub fn from_unchecked(value: impl Into<T>) -> Self {
        Self(value.into())
    }

    pub fn level(&self) -> Level {
        match self.as_ref().first().expect("valid tag") {
            b'0' => Level::Main,
            b'1' => Level::Local,
            b'2' => Level::Copy,
            _ => unreachable!(),
        }
    }
}

/// Parse a PICA+ tag.
pub fn parse_tag(i: &[u8]) -> ParseResult<&BStr> {
    map(
        recognize(tuple((
            satisfy(|c| matches!(c, '0'..='2')),
            satisfy(|c| c.is_ascii_digit()),
            satisfy(|c| c.is_ascii_digit()),
            satisfy(|c| matches!(c, 'A'..='Z' | '@')),
        ))),
        ByteSlice::as_bstr,
    )(i)
}

impl<S, T> PartialEq<Tag<S>> for Tag<T>
where
    S: AsRef<[u8]>,
    T: AsRef<[u8]>,
{
    fn eq(&self, other: &Tag<S>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T: AsRef<[u8]>> PartialEq<&str> for Tag<T> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == other.as_bytes()
    }
}

impl<T: AsRef<[u8]>> PartialEq<str> for Tag<T> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self == other
    }
}

impl<T: AsRef<[u8]>> Deref for Tag<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<[u8]>> Index<usize> for Tag<T> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index <= 3);

        let bytes = self.0.as_ref();
        &bytes[index]
    }
}

impl<T: AsRef<[u8]>> Display for Tag<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", str::from_utf8(self.0.as_ref()).unwrap())
    }
}

impl<'a> From<Tag<&'a BStr>> for TagMut {
    #[inline]
    fn from(tag: Tag<&'a BStr>) -> Self {
        Self(tag.0.into())
    }
}

impl<'a> Tag<&'a BStr> {
    /// Converts the immutable tag into its mutable counterpart by
    /// consuming the source.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let tag = TagRef::new("003@").into_owned();
    ///     assert_eq!(tag, "003@");
    ///     Ok(())
    /// }
    /// ```
    pub fn into_owned(self) -> TagMut {
        self.into()
    }

    /// Converts the immutable tag into its mutable counterpart.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let tag = TagRef::new("003@").to_owned();
    ///     assert_eq!(tag, "003@");
    ///     Ok(())
    /// }
    /// ```
    pub fn to_owned(&self) -> TagMut {
        self.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_tag_ref_new() {
        let tag = TagRef::new("003@");
        assert_eq!(tag, "003@")
    }

    #[test]
    #[should_panic(expected = "InvalidTag")]
    fn test_tag_ref_invalid() {
        TagRef::new("003!");
    }

    #[test]
    fn test_tag_mut_new() {
        let tag = TagMut::new("003@");
        assert_eq!(tag, Tag(BString::from("003@")));
        assert_eq!(tag, "003@")
    }

    #[test]
    #[should_panic(expected = "InvalidTag")]
    fn test_tag_mut_invalid() {
        TagMut::new("003!");
    }

    #[test]
    fn test_parse_tag_ref() {
        for tag in ["003@", "002@", "123@", "247C"] {
            assert_done_and_eq!(
                parse_tag(tag.as_bytes()),
                tag.as_bytes()
            )
        }

        for tag in ["456@", "0A2A", "01AA", "01Aa"] {
            assert_error!(parse_tag(tag.as_bytes()))
        }
    }

    #[test]
    fn test_display_tag() {
        assert_eq!(
            TagRef::from_bytes(b"003@").unwrap().to_string(),
            "003@".to_string()
        );
    }
}
