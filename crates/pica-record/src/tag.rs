use std::fmt::{self, Display};
use std::ops::{Deref, Index};

use bstr::{BStr, BString, ByteSlice};
use winnow::token::one_of;
use winnow::{PResult, Parser};

use crate::{Level, ParsePicaError};

/// An immutable PICA+ tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRef<'a>(&'a BStr);

/// A mutable PICA+ tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag(BString);

impl<'a> TagRef<'a> {
    /// Create a new immutable PICA+ tag.
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
    #[inline]
    pub fn new<B: ?Sized + AsRef<[u8]>>(value: &'a B) -> Self {
        Self::try_from(value.as_ref()).expect("valid tag")
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
    #[inline]
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_tag
            .parse(bytes)
            .map_err(|_| ParsePicaError::InvalidTag)
    }

    /// Returns the `Level` of the tag.
    ///
    /// ```rust
    /// use pica_record::{Level, TagRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert_eq!(TagRef::new("003@").level(), Level::Main);
    ///     assert_eq!(TagRef::new("101@").level(), Level::Local);
    ///     assert_eq!(TagRef::new("203@").level(), Level::Copy);
    ///     Ok(())
    /// }
    /// ```
    pub fn level(&self) -> Level {
        match self.0[0] {
            b'0' => Level::Main,
            b'1' => Level::Local,
            b'2' => Level::Copy,
            _ => unreachable!(),
        }
    }
}

impl<'a, T: AsRef<[u8]>> PartialEq<T> for TagRef<'a> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl<'a> Deref for TagRef<'a> {
    type Target = BStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Index<usize> for TagRef<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.0.len());
        &self.0[index]
    }
}

impl<'a> Display for TagRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[inline]
pub fn parse_tag<'a>(i: &mut &'a [u8]) -> PResult<TagRef<'a>> {
    (
        one_of([b'0', b'1', b'2']),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| c.is_ascii_uppercase() || c == b'@'),
    )
        .recognize()
        .map(|tag| TagRef(ByteSlice::as_bstr(tag)))
        .parse_next(i)
}

impl<'a> TryFrom<&'a [u8]> for TagRef<'a> {
    type Error = ParsePicaError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if parse_tag.parse(value).is_err() {
            return Err(ParsePicaError::InvalidTag);
        }

        Ok(Self(value.into()))
    }
}

impl Tag {
    /// Create a new mutable PICA+ tag.
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
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(value: &T) -> Self {
        TagRef::new(value).into()
    }
}

impl From<TagRef<'_>> for Tag {
    #[inline]
    fn from(value: TagRef<'_>) -> Self {
        Tag(value.0.into())
    }
}

impl PartialEq<TagRef<'_>> for Tag {
    #[inline]
    fn eq(&self, other: &TagRef<'_>) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<Tag> for TagRef<'_> {
    #[inline]
    fn eq(&self, other: &Tag) -> bool {
        other.0 == self.0
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for Tag {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tag() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    super::parse_tag.parse($input).unwrap(),
                    $expected
                );
            };
        }

        for tag in [b"003@", b"002@", b"123@", b"247C"] {
            parse_success!(tag, TagRef(tag.as_bstr()));
        }

        for tag in ["456@", "0A2A", "01AA", "01Aa", "003@0"] {
            assert!(super::parse_tag.parse(tag.as_bytes()).is_err());
        }
    }
}
