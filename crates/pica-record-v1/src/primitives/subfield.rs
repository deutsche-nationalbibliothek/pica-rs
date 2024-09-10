use std::io::{self, Write};
use std::iter;
use std::ops::Deref;
use std::str::Utf8Error;

use bstr::ByteSlice;
use winnow::Parser;

use super::parse::parse_subfield_ref;
use crate::{PicaError, SubfieldCode, SubfieldValue, SubfieldValueRef};

/// An immutable PICA+ subfield.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubfieldRef<'a> {
    pub(super) code: SubfieldCode,
    pub(super) value: SubfieldValueRef<'a>,
}

impl<'a> SubfieldRef<'a> {
    /// Create a new immutable PICA+ subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::new('a', "bcd")?;
    /// assert_eq!(subfield.code(), 'a');
    /// assert_eq!(subfield.value(), "bcd");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<T>(code: char, value: &'a T) -> Result<Self, PicaError>
    where
        T: ?Sized + AsRef<[u8]>,
    {
        Ok(Self {
            code: SubfieldCode::new(code)?,
            value: SubfieldValueRef::new(value)?,
        })
    }

    /// Creates an immutable PICA+ subfield from a byte slice.
    ///
    /// # Error
    ///
    /// If an invalid subfield is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::from_bytes(b"\x1f0123456789X")?;
    /// assert_eq!(subfield.code(), '0');
    /// assert_eq!(subfield.value(), "123456789X");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(
        bytes: &'a T,
    ) -> Result<Self, PicaError> {
        let bytes = bytes.as_ref();

        parse_subfield_ref.parse(bytes).map_err(|_| {
            PicaError::InvalidSubfield(bytes.to_str_lossy().to_string())
        })
    }

    /// Returns the code of the subfield.
    #[inline]
    pub fn code(&self) -> &SubfieldCode {
        &self.code
    }

    /// Returns the value of the subfield.
    #[inline]
    pub fn value(&self) -> &SubfieldValueRef {
        &self.value
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the subfield
    /// value contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::SubfieldRef;
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
        if self.value.is_ascii() {
            return Ok(());
        }

        std::str::from_utf8(&self.value)?;
        Ok(())
    }

    /// Write the subfield into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record_v1::SubfieldRef;
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
        write!(out, "\x1f{}{}", self.code, self.value)
    }
}

impl<'a> Deref for SubfieldRef<'a> {
    type Target = SubfieldValueRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T> TryFrom<(char, &'a T)> for SubfieldRef<'a>
where
    T: AsRef<[u8]> + ?Sized,
{
    type Error = PicaError;

    fn try_from(value: (char, &'a T)) -> Result<Self, Self::Error> {
        Ok(Self {
            value: SubfieldValueRef::new(value.1)?,
            code: SubfieldCode::new(value.0)?,
        })
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
    /// use pica_record_v1::SubfieldRef;
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

/// A mutable PICA+ subfield.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subfield {
    code: SubfieldCode,
    value: SubfieldValue,
}

impl Subfield {
    /// Write the subfield into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record_v1::{Subfield, SubfieldRef};
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let subfield: Subfield =
    ///     SubfieldRef::new('0', "123456789X")?.into();
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
        write!(out, "\x1f{}{}", self.code, self.value)
    }
}

impl PartialEq<Subfield> for SubfieldRef<'_> {
    #[inline]
    fn eq(&self, other: &Subfield) -> bool {
        self.code == other.code && self.value == other.value
    }
}

impl PartialEq<SubfieldRef<'_>> for Subfield {
    #[inline]
    fn eq(&self, other: &SubfieldRef<'_>) -> bool {
        self.code == other.code && self.value == other.value
    }
}

impl From<SubfieldRef<'_>> for Subfield {
    #[inline]
    fn from(other: SubfieldRef<'_>) -> Self {
        Subfield {
            value: other.value.into(),
            code: other.code,
        }
    }
}

#[cfg(feature = "arbitrary")]
impl quickcheck::Arbitrary for Subfield {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            code: SubfieldCode::arbitrary(g),
            value: SubfieldValue::arbitrary(g),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_subfield_ref_new() {
        let subfield = SubfieldRef::new('a', "abc").unwrap();
        assert_eq!(subfield.code(), 'a');
        assert_eq!(subfield.value(), "abc");

        assert_eq!(
            SubfieldRef::new('!', "abc").unwrap_err(),
            PicaError::InvalidSubfieldCode('!')
        );

        assert_eq!(
            SubfieldRef::new('a', "a\x1fbc").unwrap_err(),
            PicaError::InvalidSubfieldValue("a\x1fbc".to_string())
        );
    }

    #[test]
    fn test_subfield_ref_from_bytes() {
        let subfield = SubfieldRef::from_bytes(b"\x1f0abc").unwrap();
        assert_eq!(subfield.code(), '0');
        assert_eq!(subfield.value(), "abc");

        assert_eq!(
            SubfieldRef::from_bytes("\x1f!abc").unwrap_err(),
            PicaError::InvalidSubfield("\x1f!abc".to_string())
        );
    }

    #[test]
    fn test_subfield_ref_code() {
        let subfield = SubfieldRef::new('1', "abc").unwrap();
        assert_eq!(subfield.code(), '1');
    }

    #[test]
    fn test_subfield_ref_value() {
        let subfield = SubfieldRef::new('1', "abc").unwrap();
        assert_eq!(subfield.value(), "abc");
    }

    #[test]
    fn test_subfield_ref_is_empty() {
        let subfield = SubfieldRef::new('1', "abc").unwrap();
        assert!(!subfield.is_empty());

        let subfield = SubfieldRef::new('1', "").unwrap();
        assert!(subfield.is_empty());
    }

    #[test]
    fn test_subfield_ref_validate() {
        let subfield = SubfieldRef::new('1', "abc").unwrap();
        assert!(subfield.validate().is_ok());

        let subfield =
            SubfieldRef::from_bytes(&[b'\x1f', b'0', 0, 159]).unwrap();
        assert!(subfield.validate().is_err());
    }

    #[test]
    fn test_subfield_ref_write_to() {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield = SubfieldRef::new('0', "abcdef").unwrap();
        let _ = subfield.write_to(&mut writer);
        assert_eq!(writer.into_inner(), b"\x1f0abcdef");
    }

    #[test]
    fn test_subfield_ref_try_from() {
        let subfield = SubfieldRef::try_from(('a', "abc")).unwrap();
        assert_eq!(subfield.code(), 'a');
        assert_eq!(subfield.value(), "abc");

        let err = SubfieldRef::try_from(('!', "abc")).unwrap_err();
        assert!(matches!(err, PicaError::InvalidSubfieldCode(_)));

        let err = SubfieldRef::try_from(('a', "a\x1fc")).unwrap_err();
        assert!(matches!(err, PicaError::InvalidSubfieldValue(_)));

        let err = SubfieldRef::try_from(('a', "a\x1ec")).unwrap_err();
        assert!(matches!(err, PicaError::InvalidSubfieldValue(_)));
    }

    #[test]
    fn test_subfield_ref_into_iter() {
        let subfield = SubfieldRef::new('0', "abcdef").unwrap();
        let mut iter = subfield.into_iter();
        assert_eq!(iter.next(), Some(&subfield));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_subfield_write_to() {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield: Subfield =
            SubfieldRef::new('0', "abcdef").unwrap().into();
        let _ = subfield.write_to(&mut writer);
        assert_eq!(writer.into_inner(), b"\x1f0abcdef");
    }

    #[test]
    fn test_subfield_from_ref() {
        let subfield_ref = SubfieldRef::new('0', "abc").unwrap();
        let _subfield = Subfield::from(subfield_ref.clone());
    }
}
