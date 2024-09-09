use bstr::{BStr, BString};
use winnow::prelude::*;
use winnow::token::{one_of, take_till};

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
#[allow(unused)]
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
    pub fn new(value: &'a str) -> Result<Self, ParsePicaError> {
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

impl<T: AsRef<[u8]>> PartialEq<T> for SubfieldValueRef<'_> {
    /// Compare a [SubfieldValueRef] with any type which can be handled
    /// as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldValueRef;
    ///
    /// let value = SubfieldValueRef::new("abc")?;
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

/// Parse a PICA+ subfield value reference.
#[allow(unused)]
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

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;
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
}
