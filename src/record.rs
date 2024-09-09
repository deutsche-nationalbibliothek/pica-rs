use std::io::{self, Write};
use std::iter;
use std::ops::Deref;
use std::str::Utf8Error;

use bstr::{BStr, BString, ByteSlice};
use winnow::combinator::preceded;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{one_of, take_till, take_while};

use crate::error::ParsePicaError;

/// A subfield code.
///
/// This type represents a PICA+ subfield code, which is a single ASCII
/// alpha-numeric character.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SubfieldCode(char);

impl SubfieldCode {
    /// Creates a new subfield code.
    ///
    /// # Errors
    ///
    /// This function fails if the given code is not an ASCII
    /// alpha-numeric character.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code, 'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(code: char) -> Result<Self, ParsePicaError> {
        if !code.is_ascii_alphanumeric() {
            return Err(ParsePicaError::SubfieldCode(code));
        };

        Ok(Self(code))
    }

    /// Creates a subfied code without checking for validity.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that the given subfield code is valid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldCode;
    ///
    /// let code = SubfieldCode::from_unchecked('a');
    /// assert_eq!(code, 'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_unchecked<T: Into<char>>(code: T) -> Self {
        Self(code.into())
    }

    /// Returns the subfield code as a byte (`u8`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code.as_byte(), b'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl Deref for SubfieldCode {
    type Target = char;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<char> for SubfieldCode {
    /// Compares a [SubfieldCode] with a [char](std::char).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code, 'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, code: &char) -> bool {
        self.0 == *code
    }
}

impl PartialEq<char> for &SubfieldCode {
    fn eq(&self, other: &char) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for SubfieldCode {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        (1..)
            .map(|_| char::arbitrary(g))
            .find(char::is_ascii_alphanumeric)
            .map(SubfieldCode::from_unchecked)
            .unwrap()
    }
}

/// Parses a [SubfieldCode] from a byte slice.
pub(crate) fn parse_subfield_code(
    i: &mut &[u8],
) -> PResult<SubfieldCode> {
    one_of((b'0'..=b'9', b'a'..=b'z', b'A'..=b'Z'))
        .map(SubfieldCode::from_unchecked)
        .parse_next(i)
}

/// An immutable subfield value.
///
/// This type behaves like byte slice but guarantees that the subfield
/// value does contain neither '\x1e' nor '\x1f'.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SubfieldValueRef<'a>(
    #[cfg_attr(feature = "serde", serde(borrow))] &'a BStr,
);

impl<'a> SubfieldValueRef<'a> {
    /// Create a new subfield value reference from a byte slice.
    ///
    /// # Errors
    ///
    /// This function fails if the subfield value contains either the
    /// field separator '\x1f' or the record separator '\x1e'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::new("abc")?;
    /// assert_eq!(value, b"abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<T: AsRef<str> + ?Sized>(
        value: &'a T,
    ) -> Result<Self, ParsePicaError> {
        let value = value.as_ref();
        if value.contains('\x1f') || value.contains('\x1e') {
            return Err(ParsePicaError::SubfieldValue(value.into()));
        }

        Ok(Self(value.into()))
    }

    /// Create a new [SubfieldValueRef] from a byte slice without
    /// checking for validity.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that the value neither contains the
    /// record separator '\x1e' nor the field separator '\x1f'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::from_unchecked("abc");
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_unchecked<T>(value: &'a T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self(value.as_ref().into())
    }

    /// Creates a [SubfieldValueRef] from a byte slice.
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::from_bytes(b"abc")?;
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(
        bytes: &'a T,
    ) -> Result<Self, ParsePicaError> {
        let bytes = bytes.as_ref();

        parse_subfield_value_ref.parse(bytes).map_err(|_| {
            ParsePicaError::SubfieldValue(
                bytes.to_str_lossy().to_string(),
            )
        })
    }

    /// Returns the subfield value reference as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::from_unchecked("abc");
    /// assert_eq!(value.as_bytes(), b"abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl<'a> Deref for SubfieldValueRef<'a> {
    type Target = BStr;

    fn deref(&self) -> &'a Self::Target {
        &self.0
    }
}

impl PartialEq<str> for SubfieldValueRef<'_> {
    fn eq(&self, value: &str) -> bool {
        self.0 == value.as_bytes()
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for SubfieldValueRef<'_> {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

/// Parse a PICA+ subfield value reference.
pub(crate) fn parse_subfield_value_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldValueRef<'a>> {
    take_till(0.., |c| c == b'\x1f' || c == b'\x1e')
        .map(SubfieldValueRef::from_unchecked)
        .parse_next(i)
}

/// A mutable subfield value.
///
/// This type behaves like byte slice but guarantees that the subfield
/// value does not contain a '\x1e' or '\x1f' character.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SubfieldValue(BString);

impl SubfieldValue {
    /// Create a new subfield value from a byte slice.
    ///
    /// # Errors
    ///
    /// This function fails if the subfield value contains either the
    /// field separator '\x1f' or the record separator '\x1e'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValue;
    ///
    /// let value = SubfieldValue::new("abc")?;
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(value: &str) -> Result<Self, ParsePicaError> {
        Ok(Self::from(SubfieldValueRef::new(value)?))
    }

    /// Create a new subfield value from a byte slice without checking
    /// for validity.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that the value neither contains the
    /// record separator '\x1e' nor the field separator '\x1f'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValue;
    ///
    /// let value = SubfieldValue::from_unchecked("abc");
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_unchecked<T>(value: &T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self(BString::from(value.as_ref()))
    }

    /// Returns the subfield value as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::from_unchecked("abc");
    /// assert_eq!(value.as_bytes(), b"abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for SubfieldValue {
    type Target = BString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<SubfieldValueRef<'_>> for SubfieldValue {
    /// Creates a [SubfieldValue] from a [SubfieldValueRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{SubfieldValue, SubfieldValueRef};
    ///
    /// let value_ref = SubfieldValueRef::new("abc")?;
    /// let value = SubfieldValue::from(value_ref);
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn from(value: SubfieldValueRef<'_>) -> Self {
        Self::from_unchecked(value.0)
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for SubfieldValue {
    /// Compare a [SubfieldValue] with any type which can be handled
    /// as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValue;
    ///
    /// let value = SubfieldValue::new("abc")?;
    /// assert_eq!(value, [b'a', b'b', b'c']);
    /// assert_ne!(value, "def");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl PartialEq<SubfieldValueRef<'_>> for SubfieldValue {
    /// Compare a [SubfieldValue] with a [SubfieldValueRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{SubfieldValue, SubfieldValueRef};
    ///
    /// let value_ref = SubfieldValueRef::new("abc")?;
    /// let value = SubfieldValue::new("abc")?;
    /// assert_eq!(value_ref, value);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &SubfieldValueRef<'_>) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<str> for SubfieldValue {
    fn eq(&self, value: &str) -> bool {
        self.0 == value.as_bytes()
    }
}

impl PartialEq<SubfieldValue> for SubfieldValueRef<'_> {
    /// Compare a [SubfieldValueRef] with a [SubfieldValue].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{SubfieldValue, SubfieldValueRef};
    ///
    /// let value = SubfieldValue::new("abc")?;
    /// let value_ref = SubfieldValueRef::new("abc")?;
    /// assert_eq!(value, value_ref);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &SubfieldValue) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for SubfieldValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let value = String::arbitrary(g).replace(['\x1f', '\x1e'], "");
        Self::from_unchecked(&value)
    }
}

/// An immutable subfield.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubfieldRef<'a>(SubfieldCode, SubfieldValueRef<'a>);

impl<'a> SubfieldRef<'a> {
    /// Create a new [SubfieldRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::new('a', "abc")?;
    /// assert_eq!(subfield.code(), 'a');
    /// assert_eq!(subfield.value(), "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        code: char,
        value: &'a str,
    ) -> Result<Self, ParsePicaError> {
        let value = SubfieldValueRef::new(value)?;
        let code = SubfieldCode::new(code)?;

        Ok(Self(code, value))
    }

    /// Creates a new [SubfieldRef] from a byte slice.
    ///
    /// # Errors
    ///
    /// If an invalid subfield is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::from_bytes(b"\x1f0123456789X")?;
    /// assert_eq!(subfield.code(), '0');
    /// assert_eq!(subfield.value(), "123456789X");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(
        bytes: &'a T,
    ) -> Result<Self, ParsePicaError> {
        let bytes = bytes.as_ref();

        parse_subfield_ref.parse(bytes).map_err(|_| {
            ParsePicaError::Subfield(bytes.to_str_lossy().to_string())
        })
    }

    /// Returns the code of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::new('X', "")?;
    /// assert_eq!(subfield.code(), 'X');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn code(&self) -> &SubfieldCode {
        &self.0
    }

    /// Returns the value of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::new('a', "abc")?;
    /// assert_eq!(subfield.value(), b"abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn value(&self) -> &SubfieldValueRef {
        &self.1
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the subfield
    /// value contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::new('0', "123456789X")?;
    /// assert!(subfield.validate().is_ok());
    ///
    /// let subfield = SubfieldRef::from_bytes(&[b'\x1f', b'0', 0, 159])?;
    /// assert_eq!(subfield.validate().is_err(), true);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn validate(&self) -> Result<(), Utf8Error> {
        if self.1.is_ascii() {
            return Ok(());
        }

        std::str::from_utf8(&self.1)?;
        Ok(())
    }

    /// Write the [SubfieldRef] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::SubfieldRef;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let subfield = SubfieldRef::new('0', "123456789X")?;
    /// subfield.write_to(&mut writer);
    /// #
    /// # assert_eq!(
    /// #    String::from_utf8(writer.into_inner())?,
    /// #    "\x1f0123456789X"
    /// # );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        out.write_all(&[b'\x1f', self.0.as_byte()])?;
        out.write_all(self.1.as_bytes())
    }
}

/// Parse a PICA+ subfield.
pub(crate) fn parse_subfield_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldRef<'a>> {
    preceded(b'\x1f', (parse_subfield_code, parse_subfield_value_ref))
        .map(|(code, value)| SubfieldRef(code, value))
        .parse_next(i)
}

impl<'a> IntoIterator for &'a SubfieldRef<'a> {
    type Item = &'a SubfieldRef<'a>;
    type IntoIter = iter::Once<Self::Item>;

    /// Creates an iterator from a single subfield. The iterator just
    /// returns the subfield once.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::new('0', "123456789X")?;
    /// let mut iter = subfield.into_iter();
    /// assert_eq!(iter.next(), Some(&subfield));
    /// assert_eq!(iter.next(), None);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        iter::once(self)
    }
}

/// A mutable subfield.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subfield(SubfieldCode, SubfieldValue);

impl Subfield {
    /// Create a new immutable PICA+ [SubfieldRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// let subfield = Subfield::new('a', "abc")?;
    /// assert_eq!(subfield.code(), 'a');
    /// assert_eq!(subfield.value(), "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        code: char,
        value: &str,
    ) -> Result<Self, ParsePicaError> {
        Ok(Self::from(SubfieldRef::new(code, value)?))
    }

    /// Returns the code of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// let subfield = Subfield::new('X', "")?;
    /// assert_eq!(subfield.code(), 'X');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn code(&self) -> &SubfieldCode {
        &self.0
    }

    /// Returns the value of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// let subfield = Subfield::new('a', "abc")?;
    /// assert_eq!(subfield.value(), b"abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn value(&self) -> &SubfieldValue {
        &self.1
    }

    /// Write the subfield into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::{Subfield, SubfieldRef};
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let subfield = Subfield::new('0', "123456789X")?;
    /// subfield.write_to(&mut writer);
    /// #
    /// # assert_eq!(
    /// #    String::from_utf8(writer.into_inner())?,
    /// #    "\x1f0123456789X"
    /// # );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        out.write_all(&[b'\x1f', self.0.as_byte()])?;
        out.write_all(self.1.as_bytes())
    }
}

impl From<SubfieldRef<'_>> for Subfield {
    /// Converts a [SubfieldRef] to a [Subfield].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{Subfield, SubfieldRef};
    ///
    /// let subfield_ref = SubfieldRef::new('0', "123456789X")?;
    /// let subfield = Subfield::from(subfield_ref);
    /// assert_eq!(subfield.code(), '0');
    /// assert_eq!(subfield.value(), "123456789X");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn from(subfield: SubfieldRef<'_>) -> Self {
        let SubfieldRef(code, value) = subfield;
        Self(code, value.into())
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Subfield {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(SubfieldCode::arbitrary(g), SubfieldValue::arbitrary(g))
    }
}

/// The level (main, local, copy) of a field (or tag).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Level {
    #[default]
    Main,
    Local,
    Copy,
}

/// An immutable tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRef<'a>(&'a BStr);

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
    /// use pica_record::TagRef;
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
    /// use pica_record::TagRef;
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
    /// use pica_record::TagRef;
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
            ParsePicaError::Tag(bytes.to_str_lossy().to_string())
        })
    }

    /// Returns the [Level] of the [TagRef].
    ///
    /// ```rust
    /// use pica_record::{Level, TagRef};
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
}

/// Parse a PICA+ tag.
pub(crate) fn parse_tag_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<TagRef<'a>> {
    (
        one_of([b'0', b'1', b'2']),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| c.is_ascii_digit()),
        one_of(|c: u8| c.is_ascii_uppercase() || c == b'@'),
    )
        .take()
        .map(|tag| TagRef::from_unchecked(tag))
        .parse_next(i)
}

impl PartialEq<&str> for TagRef<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

/// A mutable tag.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// use pica_record::Tag;
    ///
    /// let tag = Tag::new("003@")?;
    /// assert_eq!(tag, "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(tag: &str) -> Result<Self, ParsePicaError> {
        Ok(Self::from(TagRef::new(tag)?))
    }

    /// Retruns the [Tag] as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Tag;
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
}

impl From<TagRef<'_>> for Tag {
    /// Create a new [Tag] from a [TagRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{Tag, TagRef};
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
    /// use pica_record::Tag;
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
    #[inline]
    fn eq(&self, other: &TagRef<'_>) -> bool {
        self.0 == other.0
    }
}
impl PartialEq<Tag> for TagRef<'_> {
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

/// An immutable occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct OccurrenceRef<'a>(&'a BStr);

impl<'a> OccurrenceRef<'a> {
    /// Create a new [OccurrenceRef] from a string slice.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::new("001")?;
    /// assert_eq!(occurrence, "001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(occ: &'a str) -> Result<Self, ParsePicaError> {
        Self::from_bytes(occ.as_bytes())
    }

    /// Create a new [OccurrenceRef] without checking for validity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the occurrence is valid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::from_unchecked("001");
    /// assert_eq!(occurrence, "001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_unchecked<T: AsRef<[u8]> + ?Sized>(occ: &'a T) -> Self {
        Self(occ.as_ref().into())
    }

    /// Create a new [OccurrenceRef] from a byte slice.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::from_bytes(b"00")?;
    /// assert_eq!(occurrence, "00");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_bytes<B: AsRef<[u8]> + ?Sized>(
        occurrence: &'a B,
    ) -> Result<Self, ParsePicaError> {
        let bytes = occurrence.as_ref();
        parse_occurrence_ref.parse(bytes).map_err(|_| {
            ParsePicaError::Occurrence(bytes.to_str_lossy().to_string())
        })
    }

    /// Write the [OccurrenceRef] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::OccurrenceRef;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let occurrence = OccurrenceRef::new("01")?;
    /// occurrence.write_to(&mut writer);
    ///
    /// assert_eq!(String::from_utf8(writer.into_inner())?, "01");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "{}", self.0)
    }
}

/// Parse PICA+ occurrence occurrence.
#[inline]
pub(crate) fn parse_occurrence_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<OccurrenceRef<'a>> {
    take_while(2..=3, AsChar::is_dec_digit)
        .map(OccurrenceRef::from_unchecked)
        .parse_next(i)
}

impl PartialEq<&str> for OccurrenceRef<'_> {
    /// Compare a [OccurrenceRef] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::from_bytes(b"01")?;
    /// assert_eq!(occurrence, "01");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn eq(&self, occurrence: &&str) -> bool {
        self.0 == occurrence.as_bytes()
    }
}

/// A mutable occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Occurrence(BString);

impl Occurrence {
    /// Create a new [OccurrenceRef] from a string slice.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::new("001")?;
    /// assert_eq!(occurrence, "001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(occ: &str) -> Result<Self, ParsePicaError> {
        Ok(Self::from(OccurrenceRef::from_bytes(occ.as_bytes())?))
    }

    /// Returns the [Occurrence] as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Occurrence;
    ///
    /// let occurrence = Occurrence::new("001")?;
    /// assert_eq!(occurrence.as_bytes(), b"001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Write the [Occurrence] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::Occurrence;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let occurrence = Occurrence::new("01")?;
    /// occurrence.write_to(&mut writer);
    ///
    /// assert_eq!(String::from_utf8(writer.into_inner())?, "01");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "{}", self.0)
    }
}

impl From<OccurrenceRef<'_>> for Occurrence {
    /// Creates a [Occurrence] from a [OccurrenceRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{Occurrence, OccurrenceRef};
    ///
    /// let occ_ref = OccurrenceRef::new("001")?;
    /// let occ = Occurrence::from(occ_ref);
    /// assert_eq!(occ, "001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn from(occurrence: OccurrenceRef<'_>) -> Self {
        let OccurrenceRef(occ) = occurrence;
        Self(occ.into())
    }
}

impl PartialEq<&str> for Occurrence {
    /// Compares a [Occurrence] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Occurrence;
    ///
    /// let occ = Occurrence::new("999")?;
    /// assert_eq!(occ, "999");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

impl PartialEq<Occurrence> for OccurrenceRef<'_> {
    /// Compares a [OccurrenceRef] with [Occurrence].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{Occurrence, OccurrenceRef};
    ///
    /// let occ_ref = OccurrenceRef::new("999")?;
    /// let occ = Occurrence::new("999")?;
    /// assert_eq!(occ_ref, occ);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &Occurrence) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Occurrence {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let size = *g.choose(&[2, 3]).unwrap();
        let value = (0..size)
            .map(|_| *g.choose(b"0123456789").unwrap())
            .collect::<Vec<u8>>();

        Occurrence(value.into())
    }
}

#[cfg(test)]
mod tests {
    use bstr::{ByteSlice, ByteVec};
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn test_parse_subfield_code() {
        (u8::MIN..=u8::MAX).into_iter().for_each(|code| {
            if !code.is_ascii_alphanumeric() {
                assert!(parse_subfield_code.parse(&[code]).is_err());
            } else {
                assert_eq!(
                    parse_subfield_code.parse(&[code]).unwrap(),
                    SubfieldCode(code as char),
                )
            }
        });
    }

    #[quickcheck]
    fn test_parse_arbitrary_subfield_code(code: SubfieldCode) {
        assert_eq!(
            parse_subfield_code.parse(&[code.as_byte()]).unwrap(),
            code,
        )
    }

    #[test]
    fn test_parse_subfield_value_ref() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr, $rest:expr) => {
                let value = SubfieldValueRef::from_unchecked($expected);
                assert_eq!(
                    parse_subfield_value_ref
                        .parse_peek($input)
                        .unwrap(),
                    ($rest.as_bytes(), value)
                );
            };
        }

        parse_success!(b"abc", b"abc", b"");
        parse_success!(b"a\x1ebc", b"a", b"\x1ebc");
        parse_success!(b"a\x1fbc", b"a", b"\x1fbc");
        parse_success!(b"", b"", b"");
    }

    #[quickcheck]
    fn test_parse_arbitrary_subfield_value_ref(value: SubfieldValue) {
        assert_eq!(
            parse_subfield_value_ref.parse(value.as_bytes()).unwrap(),
            value,
        )
    }

    #[quickcheck]
    fn test_parse_arbitrary_subfield_ref(subfield: Subfield) {
        let mut bytes = Vec::new();
        subfield.write_to(&mut bytes).unwrap();

        let result = parse_subfield_ref.parse(&bytes).unwrap();
        assert_eq!(result.value(), subfield.value());
        assert_eq!(result.code(), subfield.code());
    }

    #[quickcheck]
    fn test_parse_arbitrary_tag_ref(tag: Tag) {
        let bytes = Vec::from_slice(tag.as_bytes());
        assert_eq!(parse_tag_ref.parse(&bytes).unwrap(), tag);
    }

    #[quickcheck]
    fn test_parse_arbitrary_occurrence_ref(occurrence: Occurrence) {
        let bytes = Vec::from_slice(occurrence.as_bytes());
        assert_eq!(
            parse_occurrence_ref.parse(&bytes).unwrap(),
            occurrence
        );
    }
}
