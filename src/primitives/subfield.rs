use std::fmt::{self, Display};
use std::io::{self, Write};
use std::iter;
use std::ops::Deref;
use std::str::Utf8Error;

use bstr::{BStr, BString};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use winnow::Parser;

use super::ParsePicaError;
use super::parse::{parse_subfield_ref, parse_subfield_value_ref};

/// A subfield code.
///
/// This type represents a PICA+ subfield code, which is a single ASCII
/// alpha-numeric character.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubfieldCode(pub(super) char);

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
    /// use pica_record::primitives::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code, 'a');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(code: char) -> Result<Self, ParsePicaError> {
        if !code.is_ascii_alphanumeric() {
            return Err(ParsePicaError(format!(
                "'{code}' is not a valid subfield code"
            )));
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
    /// use pica_record::primitives::SubfieldCode;
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
    /// use pica_record::primitives::SubfieldCode;
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
    /// use pica_record::primitives::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(code, 'a');
    /// assert_ne!(code, 'b');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, code: &char) -> bool {
        self.0 == *code
    }
}

impl PartialEq<char> for &SubfieldCode {
    /// Compares a [SubfieldCode] reference with a [char](std::char).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldCode;
    ///
    /// let code = SubfieldCode::new('a')?;
    /// assert_eq!(&code, 'a');
    /// assert_ne!(&code, 'b');
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.0 == *other
    }
}

impl Display for SubfieldCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

/// An immutable subfield value.
///
/// This type behaves like byte slice but guarantees that the subfield
/// value does contain neither '\x1e' nor '\x1f'.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// use pica_record::primitives::SubfieldValueRef;
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
            return Err(ParsePicaError(format!(
                "invalid subfield value '{value}'"
            )));
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
    /// use pica_record::primitives::SubfieldValueRef;
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldValueRef;
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
            ParsePicaError(format!(
                "invalid subfield value '{bytes:?}'"
            ))
        })
    }

    /// Returns the subfield value reference as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldValueRef;
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
        self.0
    }
}

impl PartialEq<str> for SubfieldValueRef<'_> {
    /// Compare a [SubfieldValueRef] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::from_unchecked("abc");
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, value: &str) -> bool {
        self.0 == value.as_bytes()
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for SubfieldValueRef<'_> {
    /// Compare a [SubfieldValueRef] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::from_unchecked("abc");
    /// assert_eq!(value, b"abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

/// A mutable subfield value.
///
/// This type behaves like byte slice but guarantees that the subfield
/// value does not contain a '\x1e' or '\x1f' character.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// use pica_record::primitives::SubfieldValue;
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
    /// use pica_record::primitives::SubfieldValue;
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
    /// use pica_record::primitives::SubfieldValueRef;
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
    /// use pica_record::primitives::{SubfieldValue, SubfieldValueRef};
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
    /// use pica_record::primitives::SubfieldValue;
    ///
    /// let value = SubfieldValue::new("abc")?;
    /// assert_eq!(value, [b'a', b'b', b'c']);
    /// assert_ne!(value, b"def");
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
    /// use pica_record::primitives::{SubfieldValue, SubfieldValueRef};
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
    /// Compare a [SubfieldValue] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldValue;
    ///
    /// let value = SubfieldValue::new("abc")?;
    /// assert_eq!(value, "abc");
    /// assert_ne!(value, "def");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
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
    /// use pica_record::primitives::{SubfieldValue, SubfieldValueRef};
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubfieldRef<'a>(
    pub(crate) SubfieldCode,
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate)  SubfieldValueRef<'a>,
);

impl<'a> SubfieldRef<'a> {
    /// Create a new [SubfieldRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldRef;
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
    /// use pica_record::primitives::SubfieldRef;
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
            ParsePicaError(format!("invalid subfield {bytes:?}"))
        })
    }

    /// Returns the code of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldRef;
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
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::new('a', "abc")?;
    /// assert_eq!(subfield.value(), b"abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn value(&self) -> &SubfieldValueRef<'_> {
        &self.1
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the subfield
    /// value contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::SubfieldRef;
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
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let subfield = SubfieldRef::new('0', "123456789X")?;
    /// subfield.write_to(&mut writer);
    ///
    /// assert_eq!(
    ///     String::from_utf8(writer.into_inner())?,
    ///     "\x1f0123456789X"
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        out.write_all(&[b'\x1f', self.0.as_byte()])?;
        out.write_all(self.1.as_bytes())
    }
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
    /// use pica_record::primitives::SubfieldRef;
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Subfield(SubfieldCode, SubfieldValue);

impl Subfield {
    /// Create a new immutable PICA+ [SubfieldRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::Subfield;
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
    /// use pica_record::primitives::Subfield;
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
    /// use pica_record::primitives::Subfield;
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
    /// use pica_record::primitives::{Subfield, SubfieldRef};
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
    /// use pica_record::primitives::{Subfield, SubfieldRef};
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

impl PartialEq<Subfield> for SubfieldRef<'_> {
    /// Compare a [SubfieldRef] with a [Subfield].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{Subfield, SubfieldRef};
    ///
    /// let subfield_ref = SubfieldRef::new('0', "123456789X")?;
    /// let subfield = Subfield::new('0', "123456789X")?;
    /// assert_eq!(subfield_ref, subfield);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &Subfield) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Subfield {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(SubfieldCode::arbitrary(g), SubfieldValue::arbitrary(g))
    }
}
