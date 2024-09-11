use std::io::{self, Write};

use bstr::{BStr, BString};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use winnow::Parser;

use super::parse::parse_tag_ref;
use super::ParsePicaError;

/// The level (main, local, copy) of a field (or tag).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Level {
    #[default]
    Main,
    Local,
    Copy,
}

/// An immutable tag.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TagRef<'a>(
    #[cfg_attr(feature = "serde", serde(borrow))] &'a BStr,
);

impl<'a> TagRef<'a> {
    /// Create a new [TagRef] from a string slice.
    ///
    /// # Panics
    ///
    /// This method panics if the tag is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::TagRef;
    ///
    /// let tag = TagRef::new("003@")?;
    /// assert_eq!(tag, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(tag: &'a str) -> Result<Self, ParsePicaError> {
        Self::from_bytes(tag.as_bytes())
    }

    /// Creates a new [TagRef] without checking for validity.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that the given tag is valid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::TagRef;
    ///
    /// let tag = TagRef::from_unchecked(b"004A");
    /// assert_eq!(tag, "004A");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_unchecked<T: AsRef<[u8]> + ?Sized>(tag: &'a T) -> Self {
        Self(tag.as_ref().into())
    }

    /// Create a new [TagRef] from a byte slice.
    ///
    /// # Panics
    ///
    /// This method panics if the tag is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::TagRef;
    ///
    /// let tag = TagRef::from_bytes(b"003@")?;
    /// assert_eq!(tag, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_bytes<B: AsRef<[u8]> + ?Sized>(
        tag: &'a B,
    ) -> Result<Self, ParsePicaError> {
        let bytes = tag.as_ref();

        parse_tag_ref.parse(bytes).map_err(|_| {
            ParsePicaError(format!("invalid tag '{bytes:?}'"))
        })
    }

    /// Returns the [Level] of the [TagRef].
    ///
    /// ```rust
    /// use pica_record::primitives::{Level, TagRef};
    ///
    /// assert_eq!(TagRef::new("003@")?.level(), Level::Main);
    /// assert_eq!(TagRef::new("101@")?.level(), Level::Local);
    /// assert_eq!(TagRef::new("203@")?.level(), Level::Copy);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn level(&self) -> Level {
        match self.0[0] {
            b'0' => Level::Main,
            b'1' => Level::Local,
            b'2' => Level::Copy,
            _ => unreachable!(),
        }
    }

    /// Write the [TagRef] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::primitives::TagRef;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let tag = TagRef::new("003@")?;
    /// tag.write_to(&mut writer);
    ///
    /// assert_eq!(String::from_utf8(writer.into_inner())?, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        out.write_all(self.0)
    }
}

impl PartialEq<&str> for TagRef<'_> {
    /// Compare a [TagRef] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::TagRef;
    ///
    /// let tag = TagRef::new("003@")?;
    /// assert_eq!(tag, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

impl PartialEq<&str> for &TagRef<'_> {
    /// Compare a [TagRef] reference with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::TagRef;
    ///
    /// let tag = TagRef::new("003@")?;
    /// assert_eq!(&tag, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

/// A mutable tag.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tag(BString);

impl Tag {
    /// Create a new [Tag].
    ///
    /// # Errors
    ///
    /// This function returns an error if the given tag is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::Tag;
    ///
    /// let tag = Tag::new("003@")?;
    /// assert_eq!(tag, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(tag: &str) -> Result<Self, ParsePicaError> {
        Ok(Self::from(TagRef::new(tag)?))
    }

    /// Returns the [Tag] as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::Tag;
    ///
    /// let tag = Tag::new("003@")?;
    /// assert_eq!(tag.as_bytes(), b"003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Write the [Tag] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::primitives::Tag;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let tag = Tag::new("003@")?;
    /// tag.write_to(&mut writer);
    ///
    /// assert_eq!(String::from_utf8(writer.into_inner())?, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        out.write_all(&self.0)
    }
}

impl From<TagRef<'_>> for Tag {
    /// Create a new [Tag] from a [TagRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{Tag, TagRef};
    ///
    /// let tag_ref = TagRef::new("003@")?;
    /// let tag = Tag::from(tag_ref);
    /// assert_eq!(tag, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn from(tag: TagRef<'_>) -> Self {
        let TagRef(value) = tag;
        Self(value.into())
    }
}

impl PartialEq<&str> for Tag {
    /// Compares a [Tag] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::Tag;
    ///
    /// let tag = Tag::new("003@")?;
    /// assert_eq!(tag, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

impl PartialEq<TagRef<'_>> for Tag {
    /// Compares a [Tag] with a [TagRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{Tag, TagRef};
    ///
    /// let tag = Tag::new("003@")?;
    /// let tag_ref = Tag::new("003@")?;
    /// assert_eq!(tag, tag_ref);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &TagRef<'_>) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<Tag> for TagRef<'_> {
    /// Compares a [TagRef] with a [Tag].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{Tag, TagRef};
    ///
    /// let tag_ref = Tag::new("003@")?;
    /// let tag = Tag::new("003@")?;
    /// assert_eq!(tag_ref, tag);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &Tag) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Tag {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let p0 = *g.choose(b"012").unwrap();
        let p1 = *g.choose(b"0123456789").unwrap();
        let p2 = *g.choose(b"0123456789").unwrap();
        let p3 = *g.choose(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ@").unwrap();

        Self(BString::from(&[p0, p1, p2, p3]))
    }
}
