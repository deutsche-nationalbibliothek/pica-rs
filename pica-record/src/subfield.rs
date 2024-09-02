use std::fmt::{self, Display};
use std::io::{self, Write};
use std::iter;
use std::ops::Deref;
use std::str::Utf8Error;

use bstr::{BStr, ByteSlice};
use winnow::combinator::preceded;
use winnow::token::{one_of, take_till};
use winnow::{PResult, Parser};

use crate::PicaError;

/// A PICA+ subfield code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SubfieldCode(char);

impl SubfieldCode {
    /// Creates a new subfield code.
    ///
    /// # Error
    ///
    /// This functions fails if the given code is not an ASCII
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
    pub fn new(code: char) -> Result<Self, PicaError> {
        if !code.is_ascii_alphanumeric() {
            return Err(PicaError::InvalidSubfieldCode(code));
        }

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

impl Display for SubfieldCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<char> for SubfieldCode {
    fn eq(&self, code: &char) -> bool {
        self.0 == *code
    }
}

impl PartialEq<char> for &SubfieldCode {
    fn eq(&self, code: &char) -> bool {
        self.0 == *code
    }
}

impl TryFrom<char> for SubfieldCode {
    type Error = PicaError;

    fn try_from(code: char) -> Result<Self, Self::Error> {
        Self::new(code)
    }
}

#[cfg(feature = "arbitrary")]
impl quickcheck::Arbitrary for SubfieldCode {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let code = (1..)
            .map(|_| char::arbitrary(g))
            .find(char::is_ascii_alphanumeric)
            .unwrap();

        Self(code)
    }
}

/// Parse a PICA+ subfield code.
pub fn parse_subfield_code(i: &mut &[u8]) -> PResult<SubfieldCode> {
    one_of((b'0'..=b'9', b'a'..=b'z', b'A'..=b'Z'))
        .map(SubfieldCode::from_unchecked)
        .parse_next(i)
}

/// An immutable PICA+ subfield value.
///
/// This type behaves like byte slice but guarantees that the subfield
/// value does not contain neither '\x1e' or '\x1f'.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SubfieldValueRef<'a>(&'a [u8]);

impl<'a> SubfieldValueRef<'a> {
    /// Create a new subfield value reference from a byte slice.
    ///
    /// # Error
    ///
    /// This function fails if the subfield value contains either the
    /// field separator '\x1f' or the record separator '\x1e'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::new(b"abc")?;
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<T>(value: &'a T) -> Result<Self, PicaError>
    where
        T: AsRef<[u8]> + ?Sized,
    {
        let value = value.as_ref();
        if value.find_byteset(b"\x1f\x1e").is_some() {
            return Err(PicaError::InvalidSubfieldValue(
                value.to_str_lossy().to_string(),
            ));
        }

        Ok(Self(value))
    }

    /// Create a new subfield value reference from a byte slice without
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
    pub fn from_unchecked<T>(value: &'a T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self(value.as_ref())
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
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }
}

impl<'a> Deref for SubfieldValueRef<'a> {
    type Target = BStr;

    fn deref(&self) -> &Self::Target {
        self.0.as_bstr()
    }
}

impl Display for SubfieldValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_bstr())
    }
}

impl PartialEq<str> for SubfieldValueRef<'_> {
    fn eq(&self, value: &str) -> bool {
        self.0 == value.as_bytes()
    }
}

impl PartialEq<&str> for SubfieldValueRef<'_> {
    fn eq(&self, value: &&str) -> bool {
        self.0 == value.as_bytes()
    }
}

impl PartialEq<Vec<u8>> for SubfieldValueRef<'_> {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0 == other
    }
}

/// Parse a PICA+ subfield value reference.
pub fn parse_subfield_value_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldValueRef<'a>> {
    take_till(0.., |c| c == b'\x1f' || c == b'\x1e')
        .map(SubfieldValueRef)
        .parse_next(i)
}

/// A mutable PICA+ subfield value.
///
/// This type behaves like byte slice but guarantees that the subfield
/// value does not contain neither '\x1e' or '\x1f'.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SubfieldValue(Vec<u8>);

impl SubfieldValue {
    /// Create a new subfield value from a byte slice.
    ///
    /// # Error
    ///
    /// This function fails if the subfield value contains either the
    /// field separator '\x1f' or the record separator '\x1e'.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValue;
    ///
    /// let value = SubfieldValue::new(b"abc")?;
    /// assert_eq!(value, "abc");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<T>(value: &T) -> Result<Self, PicaError>
    where
        T: AsRef<[u8]>,
    {
        let value = value.as_ref();
        if value.find_byteset(b"\x1f\x1e").is_some() {
            return Err(PicaError::InvalidSubfieldValue(
                value.to_str_lossy().to_string(),
            ));
        }

        Ok(Self(value.to_vec()))
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
        Self(value.as_ref().to_vec())
    }
}

impl Display for SubfieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_bstr())
    }
}

impl From<SubfieldValueRef<'_>> for SubfieldValue {
    fn from(value: SubfieldValueRef<'_>) -> Self {
        Self(value.to_vec())
    }
}

impl PartialEq<SubfieldValueRef<'_>> for SubfieldValue {
    fn eq(&self, other: &SubfieldValueRef<'_>) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<SubfieldValue> for SubfieldValueRef<'_> {
    fn eq(&self, other: &SubfieldValue) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<&str> for SubfieldValue {
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

#[cfg(feature = "arbitrary")]
impl quickcheck::Arbitrary for SubfieldValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let value = String::arbitrary(g).replace(['\x1f', '\x1e'], "");
        Self::from_unchecked(&value)
    }
}

/// An immutable PICA+ subfield.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubfieldRef<'a> {
    code: SubfieldCode,
    value: SubfieldValueRef<'a>,
}

impl<'a> SubfieldRef<'a> {
    /// Create a new immutable PICA+ subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
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
    /// use pica_record::{Subfield, SubfieldRef};
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

/// Parse a PICA+ subfield.
#[inline]
pub(crate) fn parse_subfield_ref<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldRef<'a>> {
    preceded(b'\x1f', (parse_subfield_code, parse_subfield_value_ref))
        .map(|(code, value)| SubfieldRef { code, value })
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use io::Cursor;

    use super::*;

    #[test]
    fn test_subfield_code_new() {
        for c in '0'..='z' {
            if c.is_ascii_alphanumeric() {
                assert_eq!(
                    SubfieldCode::new(c).unwrap(),
                    SubfieldCode(c)
                );
            } else {
                assert_eq!(
                    SubfieldCode::new(c).unwrap_err(),
                    PicaError::InvalidSubfieldCode(c)
                );
            }
        }
    }

    #[test]
    fn test_subfield_code_from_unchecked() {
        for c in '0'..='z' {
            if c.is_ascii_alphanumeric() {
                assert_eq!(
                    SubfieldCode::from_unchecked(c),
                    SubfieldCode(c)
                );
            }
        }
    }

    #[test]
    fn test_subfield_code_as_byte() {
        for c in '0'..='z' {
            if c.is_ascii_alphanumeric() {
                let code = SubfieldCode::new(c).unwrap();
                assert_eq!(code.as_byte(), c as u8);
            }
        }
    }

    #[test]
    fn test_subfield_code_try_from_char() {
        for c in '0'..='z' {
            if c.is_ascii_alphanumeric() {
                assert_eq!(
                    SubfieldCode::try_from(c).unwrap(),
                    SubfieldCode(c)
                );
            } else {
                assert_eq!(
                    SubfieldCode::try_from(c).unwrap_err(),
                    PicaError::InvalidSubfieldCode(c)
                );
            }
        }
    }

    #[test]
    fn test_parse_subfield_code() {
        for c in b'0'..=b'z' {
            if c.is_ascii_alphanumeric() {
                assert_eq!(
                    parse_subfield_code.parse(&[c]).unwrap(),
                    SubfieldCode::from_unchecked(c)
                );
            } else {
                assert!(parse_subfield_code.parse(&[c]).is_err());
            }
        }
    }

    #[test]
    fn test_subfield_value_ref_new() {
        let value = SubfieldValueRef::new("abc").unwrap();
        assert_eq!(value, "abc");

        let value = SubfieldValueRef::new("").unwrap();
        assert_eq!(value, "");

        assert_eq!(
            SubfieldValueRef::new("abc\x1e").unwrap_err(),
            PicaError::InvalidSubfieldValue("abc\x1e".to_string())
        );

        assert_eq!(
            SubfieldValueRef::new("abc\x1f").unwrap_err(),
            PicaError::InvalidSubfieldValue("abc\x1f".to_string())
        );
    }

    #[test]
    fn test_subfield_value_ref_from_unchecked() {
        let value = SubfieldValueRef::from_unchecked("abc");
        assert_eq!(value, "abc");

        let value = SubfieldValueRef::from_unchecked("");
        assert_eq!(value, "");
    }

    #[test]
    fn test_subfield_value_ref_as_bytes() {
        let value = SubfieldValueRef::from_unchecked("abc");
        assert_eq!(value.as_bytes(), b"abc");
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

    #[test]
    fn subfield_ref_new() {
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
    fn subfield_ref_from_bytes() {
        let subfield = SubfieldRef::from_bytes(b"\x1f0abc").unwrap();
        assert_eq!(subfield.code(), '0');
        assert_eq!(subfield.value(), "abc");

        assert_eq!(
            SubfieldRef::from_bytes("\x1f!abc").unwrap_err(),
            PicaError::InvalidSubfield("\x1f!abc".to_string())
        );
    }

    #[test]
    fn subfield_ref_code() {
        let subfield = SubfieldRef::new('1', "abc").unwrap();
        assert_eq!(subfield.code(), '1');
    }

    #[test]
    fn subfield_ref_value() {
        let subfield = SubfieldRef::new('1', "abc").unwrap();
        assert_eq!(subfield.value(), "abc");
    }

    #[test]
    fn subfield_ref_is_empty() {
        let subfield = SubfieldRef::new('1', "abc").unwrap();
        assert!(!subfield.is_empty());

        let subfield = SubfieldRef::new('1', "").unwrap();
        assert!(subfield.is_empty());
    }

    #[test]
    fn subfield_ref_validate() {
        let subfield = SubfieldRef::new('1', "abc").unwrap();
        assert!(subfield.validate().is_ok());

        let subfield =
            SubfieldRef::from_bytes(&[b'\x1f', b'0', 0, 159]).unwrap();
        assert!(subfield.validate().is_err());
    }

    #[test]
    fn subfield_ref_write_to() {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield = SubfieldRef::new('0', "abcdef").unwrap();
        let _ = subfield.write_to(&mut writer);
        assert_eq!(writer.into_inner(), b"\x1f0abcdef");
    }

    #[test]
    fn subfield_ref_try_from() {
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
    fn subfield_ref_into_iter() {
        let subfield = SubfieldRef::new('0', "abcdef").unwrap();
        let mut iter = subfield.into_iter();

        assert_eq!(iter.next(), Some(&subfield));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield_ref.parse(b"\x1fa123").unwrap(),
            SubfieldRef::new('a', "123").unwrap()
        );

        assert_eq!(
            parse_subfield_ref.parse(b"\x1fa").unwrap(),
            SubfieldRef::new('a', "").unwrap()
        );

        assert!(parse_subfield_ref.parse(b"a123").is_err());
        assert!(parse_subfield_ref.parse(b"").is_err());
    }

    #[cfg_attr(miri, ignore)]
    #[quickcheck_macros::quickcheck]
    fn test_parse_subfield_ref_arbitrary(subfield: Subfield) -> bool {
        let mut bytes = Vec::<u8>::new();
        let _ = subfield.write_to(&mut bytes);
        parse_subfield_ref.parse(&bytes).is_ok()
    }

    #[test]
    fn subfield_write_to() {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield: Subfield =
            SubfieldRef::new('0', "abcdef").unwrap().into();
        let _ = subfield.write_to(&mut writer);
        assert_eq!(writer.into_inner(), b"\x1f0abcdef");
    }

    #[test]
    fn subfield_from_ref() {
        let subfield_ref = SubfieldRef::new('0', "abc").unwrap();
        let _subfield = Subfield::from(subfield_ref.clone());
    }
}
