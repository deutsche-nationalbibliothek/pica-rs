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
    /// use pica_record_v1::TagRef;
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
    /// use pica_record_v1::TagRef;
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
    /// use pica_record_v1::{Level, TagRef};
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

impl<T: AsRef<[u8]>> PartialEq<T> for TagRef<'_> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl PartialEq<str> for TagRef<'_> {
    /// Compare a `TagRef` with a string slice.
    ///
    /// ```rust
    /// use pica_record_v1::{Level, TagRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert_eq!(&TagRef::new("003@"), "003@");
    ///     assert_eq!(TagRef::new("003@"), "003@");
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl Deref for TagRef<'_> {
    type Target = BStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Index<usize> for TagRef<'_> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.0.len());
        &self.0[index]
    }
}

impl Display for TagRef<'_> {
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
        .take()
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
    /// use pica_record_v1::Tag;
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

    /// Returns the tag as an byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let tag = Tag::new("003@");
    ///     assert_eq!(tag.as_bytes(), b"003@");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
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

#[cfg(feature = "arbitrary")]
impl quickcheck::Arbitrary for Tag {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let p0 = *g.choose(b"012").unwrap();
        let p1 = *g.choose(b"0123456789").unwrap();
        let p2 = *g.choose(b"0123456789").unwrap();
        let p3 = *g.choose(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ@").unwrap();

        let inner = BString::from(&[p0, p1, p2, p3]);

        Tag(inner)
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

    #[quickcheck_macros::quickcheck]
    fn parse_arbitrary_tag(tag: Tag) -> bool {
        super::parse_tag.parse(tag.as_bytes()).is_ok()
    }
}
