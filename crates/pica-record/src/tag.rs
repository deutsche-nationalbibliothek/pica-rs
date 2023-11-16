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
pub fn parse_tag<'a>(i: &mut &'a [u8]) -> PResult<Tag<'a>> {
    (
        one_of([b'0', b'1', b'2']),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| (b'A'..=b'Z').contains(&c) || c == b'@'),
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
    #[inline]
    pub fn new<B: ?Sized + AsRef<[u8]>>(value: &'a B) -> Self {
        Self::try_from(value.as_ref().as_bstr()).expect("valid tag")
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

impl<'a> TryFrom<&'a BStr> for Tag<'a> {
    type Error = ParsePicaError;

    fn try_from(value: &'a BStr) -> Result<Self, Self::Error> {
        if parse_tag.parse(value).is_err() {
            return Err(ParsePicaError::InvalidTag);
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;

    use super::*;

    #[test]
    fn parse_tag() {
        use super::parse_tag;

        for tag in ["003@", "002@", "123@", "247C"] {
            assert_eq!(
                parse_tag.parse(tag.as_bytes()).unwrap(),
                Tag(tag.as_bytes().into())
            );
        }

        for tag in ["456@", "0A2A", "01AA", "01Aa", "003@0"] {
            assert!(parse_tag.parse(tag.as_bytes()).is_err());
        }
    }

    #[test]
    fn tag_new() {
        for i in ["003@", "101@", "203@"] {
            assert_eq!(Tag::new(i), Tag(i.as_bytes().as_bstr()));
        }
    }

    #[test]
    #[should_panic]
    fn tag_new_panic() {
        Tag::new("403@");
    }

    #[test]
    fn tag_from_bytes() {
        for i in ["003@", "101@", "203@"] {
            let bytes = i.as_bytes();

            assert_eq!(
                Tag::from_bytes(bytes).unwrap(),
                Tag(bytes.as_bstr())
            );
        }

        for i in ["003@0", "403@", "03@"] {
            assert_eq!(
                Tag::from_bytes(i.as_bytes()).unwrap_err(),
                ParsePicaError::InvalidTag
            );
        }
    }

    #[test]
    fn tag_try_from() {
        for i in ["003@", "101@", "203@"] {
            let bytes = i.as_bytes();

            assert_eq!(
                Tag::try_from(bytes.as_bstr()).unwrap(),
                Tag(bytes.as_bstr())
            );
        }

        for i in ["003@0", "403@", "03@"] {
            assert_eq!(
                Tag::try_from(i.as_bytes().as_bstr()).unwrap_err(),
                ParsePicaError::InvalidTag
            );
        }
    }

    #[test]
    fn tag_level() {
        let tag = Tag::new("003@");
        assert_eq!(tag.level(), Level::Main);

        let tag = Tag::new("101@");
        assert_eq!(tag.level(), Level::Local);

        let tag = Tag::new("203@");
        assert_eq!(tag.level(), Level::Copy);
    }

    #[test]
    fn tag_eq() {
        assert_eq!(Tag::new("003@"), b"003@");
        assert_eq!(Tag::new("003@"), "003@");
    }

    #[test]
    fn tag_deref() {
        let tag = Tag::new("003@");
        assert_eq!(tag.len(), 4);
    }

    #[test]
    fn tag_index() {
        let tag = Tag::new("003@");
        assert_eq!(tag[0], b'0');
        assert_eq!(tag[1], b'0');
        assert_eq!(tag[2], b'3');
        assert_eq!(tag[3], b'@');
    }

    #[test]
    #[should_panic]
    fn tag_index_panic() {
        let tag = Tag::new("003@");
        assert_eq!(tag[4], b'0');
    }

    #[test]
    fn tag_to_string() {
        let tag_str = format!("{}", Tag::new("003@"));
        assert_eq!(tag_str, "003@");
    }
}
