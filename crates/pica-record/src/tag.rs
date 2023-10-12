use std::fmt::{self, Display};
use std::ops::{Deref, Index};

use bstr::{BStr, ByteSlice};
use winnow::token::one_of;
use winnow::{PResult, Parser};

use crate::{Level, ParsePicaError};

/// An immutable PICA+ tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag<'a>(&'a BStr);

/// Parse a PICA+ tag.
#[inline]
pub(crate) fn parse_tag<'a>(i: &mut &'a [u8]) -> PResult<Tag<'a>> {
    (
        one_of([b'0', b'1', b'2']),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| matches!(c, b'A'..=b'Z' | b'@')),
    )
        .recognize()
        .map(|tag| Tag(ByteSlice::as_bstr(tag)))
        .parse_next(i)
}

impl<'a> Tag<'a> {
    /// Create a new PICA+ tag.
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
    pub fn new<B: ?Sized + AsRef<[u8]>>(value: &'a B) -> Self {
        Self::from_bytes(value.as_ref()).expect("valid tag")
    }

    /// Creates an PICA+ tag from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(Tag::from_bytes(b"003@").is_ok());
    ///     assert!(Tag::from_bytes(b"!03@").is_err());
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
    pub fn level(&self) -> Level {
        match self.0[0] {
            b'0' => Level::Main,
            b'1' => Level::Local,
            b'2' => Level::Copy,
            _ => unreachable!(),
        }
    }
}

impl<'a, T: AsRef<[u8]>> PartialEq<T> for Tag<'a> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl<'a> Deref for Tag<'a> {
    type Target = BStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Index<usize> for Tag<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.0.len());
        &self.0[index]
    }
}

impl<'a> Display for Tag<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag() {
        for tag in ["003@", "002@", "123@", "247C"] {
            assert_eq!(
                parse_tag.parse_peek(tag.as_bytes()).unwrap(),
                ("".as_bytes(), Tag(tag.as_bytes().into()))
            );
        }

        for tag in ["456@", "0A2A", "01AA", "01Aa"] {
            assert!(parse_tag.parse_peek(tag.as_bytes()).is_err());
        }
    }

    #[test]
    fn test_index() {
        let tag = Tag::new("003@");
        assert_eq!(tag[0], b'0');
        assert_eq!(tag[1], b'0');
        assert_eq!(tag[2], b'3');
        assert_eq!(tag[3], b'@');
    }

    #[test]
    #[should_panic]
    fn test_index_panic() {
        let tag = Tag::new("003@");
        assert_eq!(tag[4], b'0');
    }
}
