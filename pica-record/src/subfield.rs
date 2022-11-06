use std::fmt::Display;
use std::io::{self, Write};
use std::str::Utf8Error;

use bstr::{BStr, BString, ByteSlice};
use nom::bytes::complete::take_till;
use nom::character::complete::{char, satisfy};
use nom::combinator::map;
use nom::sequence::{pair, preceded};
use nom::Finish;

use crate::parser::{ParseResult, RS, US};
use crate::ParsePicaError;

/// A PICA+ subfield.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subfield<T: AsRef<[u8]>> {
    pub(crate) code: char,
    pub(crate) value: T,
}

/// A immutable PICA+ subfield.
pub type SubfieldRef<'a> = Subfield<&'a BStr>;

/// A mutable PICA+ subfield.
pub type SubfieldMut = Subfield<BString>;

impl<T: AsRef<[u8]>> Subfield<T> {
    /// Returns the code of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('0', "0123456789X");
    ///     assert_eq!(subfield.code(), '0');
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> char {
        self.code
    }

    /// Returns the value of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('0', "123456789X");
    ///     assert_eq!(subfield.value(), &"123456789X".as_bytes());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<'a, T: AsRef<[u8]> + From<&'a BStr> + Display> Subfield<T> {
    /// Create a new subfield.
    ///
    /// # Panics
    ///
    /// This method panics if the subfield code or the value is
    /// invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('a', "bcd");
    ///     assert_eq!(subfield.code(), 'a');
    ///     assert_eq!(subfield.value(), &"bcd".as_bytes());
    ///     Ok(())
    /// }
    /// ```
    pub fn new(code: char, value: impl Into<T>) -> Self {
        let value = value.into();
        let bytes: &[u8] = value.as_ref();

        assert!(
            code.is_ascii_alphanumeric()
                && bytes.find_byte(b'\x1e').is_none()
                && bytes.find_byte(b'\x1f').is_none()
        );

        Self { code, value }
    }

    /// Creates an immutable PICA+ subfield from a byte slice.
    ///
    /// If an invalid subfield is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::from_bytes(b"\x1f0123456789X")?;
    ///     assert_eq!(subfield.code(), '0');
    ///     assert_eq!(subfield.value(), &"123456789X".as_bytes());
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_subfield(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidSubfield)
            .map(|(_, (code, value))| Self {
                code,
                value: value.into(),
            })
    }

    /// Returns true if the subfield value is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('0', "abc");
    ///     assert!(!subfield.is_empty());
    ///
    ///     let subfield = SubfieldRef::new('0', "");
    ///     assert!(subfield.is_empty());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.value.as_ref().len() == 0
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the subfield
    /// value contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('0', "123456789X");
    ///     assert_eq!(subfield.validate().is_ok(), true);
    ///
    ///     let subfield =
    ///         SubfieldRef::from_bytes(&[b'\x1f', b'0', 0, 159])?;
    ///     assert_eq!(subfield.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), Utf8Error> {
        let bytes = self.value.as_ref();

        if bytes.is_ascii() {
            return Ok(());
        }

        std::str::from_utf8(bytes)?;
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
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let subfield = SubfieldRef::new('0', "123456789X");
    ///     subfield.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #    String::from_utf8(writer.into_inner())?,
    ///     #    "\x1f0123456789X"
    ///     # );
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "\x1f{}{}", self.code, self.value)
    }
}

impl<'a> From<Subfield<&'a BStr>> for SubfieldMut {
    #[inline]
    fn from(subfield: Subfield<&'a BStr>) -> Self {
        Self {
            code: subfield.code,
            value: subfield.value.into(),
        }
    }
}

impl<'a> Subfield<&'a BStr> {
    /// Converts the immutable subfield into its mutable counterpart by
    /// consuming the source.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield =
    ///         SubfieldRef::new('0', "0123456789X").into_owned();
    ///     assert_eq!(subfield.value(), "0123456789X");
    ///     Ok(())
    /// }
    /// ```
    pub fn into_owned(self) -> SubfieldMut {
        self.into()
    }

    /// Converts the immutable subfield into its mutable counterpart.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::new('0', "0123456789X").to_owned();
    ///     assert_eq!(subfield.value(), "0123456789X");
    ///     Ok(())
    /// }
    /// ```
    pub fn to_owned(&self) -> SubfieldMut {
        self.clone().into()
    }
}

/// Parse a PICA+ subfield code.
pub fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

/// Parse a PICA+ subfield value.
pub fn parse_subfield_value(i: &[u8]) -> ParseResult<&BStr> {
    map(take_till(|c| c == US || c == RS), ByteSlice::as_bstr)(i)
}

/// Parse a PICA+ subfield.
pub(crate) fn parse_subfield(i: &[u8]) -> ParseResult<(char, &BStr)> {
    preceded(
        char('\x1f'),
        pair(parse_subfield_code, parse_subfield_value),
    )(i)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_subfield_new() {
        let subfield = SubfieldMut::new('a', "abc");
        assert_eq!(subfield.code(), 'a');
        assert_eq!(subfield.value(), "abc");
        assert!(!subfield.is_empty());

        let subfield: Subfield<BString> = Subfield::new('a', "");
        assert!(subfield.is_empty());
    }

    #[test]
    fn test_subfield_from_bytes() {
        let subfield =
            SubfieldMut::from_bytes(b"\x1f0123456789X").unwrap();
        assert_eq!(subfield.value(), "123456789X");
        assert_eq!(subfield.code(), '0');

        assert_eq!(
            SubfieldMut::from_bytes(b"\x1faabc").unwrap(),
            Subfield::new('a', "abc")
        );

        assert_eq!(
            SubfieldMut::from_bytes(b"abc").unwrap_err(),
            ParsePicaError::InvalidSubfield,
        );
    }

    #[test]
    #[should_panic]
    fn test_subfield_invalid_code() {
        SubfieldMut::new('!', "abc");
    }

    #[test]
    #[should_panic]
    fn test_subfield_invalid_value1() {
        SubfieldMut::new('0', "\x1f");
    }

    #[test]
    #[should_panic]
    fn test_subfield_invalid_value2() {
        SubfieldMut::new('0', "\x1e");
    }

    #[test]
    fn test_subfield_write_to() -> anyhow::Result<()> {
        let subfield = SubfieldMut::new('0', "123456789X");
        let mut writer = Cursor::new(Vec::<u8>::new());
        subfield.write_to(&mut writer)?;

        assert_eq!(
            String::from_utf8(writer.into_inner())?,
            "\x1f0123456789X"
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_code() {
        for c in b'0'..=b'z' {
            if c.is_ascii_alphanumeric() {
                assert_done_and_eq!(
                    parse_subfield_code(&[c]),
                    c as char
                );
            } else {
                assert_error!(parse_subfield_code(&[c]));
            }
        }
    }

    #[test]
    fn test_parse_subfield_value() {
        assert_done_and_eq!(parse_subfield_value(b"abc"), "abc");
        assert_done_and_eq!(parse_subfield_value(b"a\x1ebc"), "a");
        assert_done_and_eq!(parse_subfield_value(b"a\x1fbc"), "a");
        assert_done_and_eq!(parse_subfield_value(b""), "");
    }

    #[test]
    fn test_parse_subfield_ref() {
        assert_done_and_eq!(
            parse_subfield(b"\x1fa123"),
            ('a', "123".into())
        );

        assert_done_and_eq!(parse_subfield(b"\x1fa"), ('a', "".into()));

        assert!(parse_subfield(b"a123").is_err());
        assert!(parse_subfield(b"").is_err());
    }
}
