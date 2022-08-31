//! This module contains data structures and functions related to
//! PICA+ subfield.

use std::io::Write;
use std::str::{FromStr, Utf8Error};
use std::{fmt, io};

use bstr::{BStr, BString, ByteSlice};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use nom::bytes::complete::take_till;
use nom::character::complete::{char, satisfy};
use nom::combinator::map;
use nom::sequence::{pair, preceded};
use nom::Finish;

use crate::parser::{RS, US};
use crate::{ParseError, ParseResult};

/// An immutable PICA+ tag.
#[derive(Debug, PartialEq, Eq)]
pub struct SubfieldRef<'a> {
    code: char,
    value: &'a BStr,
}

/// Parse a PICA+ subfield code.
#[inline]
pub fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

/// Parse a PICA+ subfield value.
#[inline]
fn parse_subfield_value<'a>(i: &'a [u8]) -> ParseResult<&'a BStr> {
    map(take_till(|c| c == US || c == RS), ByteSlice::as_bstr)(i)
}

/// Parse a PICA+ subfield.
#[inline]
pub fn parse_subfield<'a>(i: &'a [u8]) -> ParseResult<SubfieldRef<'a>> {
    map(
        preceded(
            char('\x1f'),
            pair(parse_subfield_code, parse_subfield_value),
        ),
        |(code, value)| SubfieldRef { code, value },
    )(i)
}

impl<'a> SubfieldRef<'a> {
    /// Creates an immutable PICA+ subfield from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_core::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(SubfieldRef::from_bytes(b"\x1f0123456789X").is_ok());
    ///     assert!(SubfieldRef::from_bytes(b"abc").is_err());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(match parse_subfield(data).finish() {
            Ok((_, subfield)) => subfield,
            _ => return Err(ParseError::InvalidSubfield),
        })
    }
}

/// A mutable PICA+ subfield.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subfield {
    code: char,
    value: BString,
}

impl Subfield {
    /// Creates an PICA+ subfield from a byte slice.
    ///
    /// ```rust
    /// use pica_core::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(Subfield::from_bytes(b"\x1fabc").is_ok());
    ///     assert!(Subfield::from_bytes(b"abc").is_err());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        Ok(SubfieldRef::from_bytes(data)?.into())
    }

    /// Get a reference to the subfield's code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::from_bytes(b"\x1f012383643X")?;
    ///     assert_eq!(subfield.code(), '0');
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> char {
        self.code
    }

    /// Get a reference to the subfield's value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::from_bytes(b"\x1f012283643X")?;
    ///     assert_eq!(subfield.value(), "12283643X");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &BString {
        &self.value
    }

    /// Returns `true` if the subfield value is valid UTF-8 byte sequence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::from_bytes(b"\x1f0123456789X")?;
    ///     assert_eq!(subfield.validate().is_ok(), true);
    ///
    ///     let subfield = Subfield::from_bytes(&[b'\x1f', b'0', 0, 159])?;
    ///     assert_eq!(subfield.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
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
    /// use pica_core::Subfield;
    /// use std::io::Cursor;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let subfield = Subfield::from_bytes(b"\x1f0123456789X")?;
    ///     subfield.write(&mut writer)?;
    ///
    ///     # let result = String::from_utf8(writer.into_inner())?;
    ///     # assert_eq!(result, String::from("\x1f0123456789X"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write(&self, writer: &mut dyn Write) -> Result<(), io::Error> {
        write!(writer, "\x1f{}{}", self.code, self.value)?;
        Ok(())
    }
}

impl From<SubfieldRef<'_>> for Subfield {
    fn from(subfield_ref: SubfieldRef) -> Self {
        Subfield {
            code: subfield_ref.code,
            value: subfield_ref.value.into(),
        }
    }
}

impl FromStr for Subfield {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Subfield::from_bytes(s.as_bytes())
    }
}

impl fmt::Display for Subfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}{}", self.code, self.value)
    }
}

impl Serialize for Subfield {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Subfield", 2)?;
        state.serialize_field("tag", &self.code)?;
        // SAFETY: It's save because `Serialize` is only implemented for
        // `StringRecord` and not for `ByteRecord`.
        unsafe {
            state.serialize_field("value", &self.value.to_str_unchecked())?;
        }
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TestResult;
    use bstr::ByteSlice;
    use std::io::Cursor;

    use nom_test_helpers::prelude::*;

    #[test]
    fn test_parse_subfield_code() -> TestResult {
        for c in b'a'..=b'z' {
            assert_done_and_eq!(parse_subfield_code(&[c]), c as char);
        }

        for c in b'A'..=b'Z' {
            assert_done_and_eq!(parse_subfield_code(&[c]), c as char);
        }

        for c in b'0'..=b'9' {
            assert_done_and_eq!(parse_subfield_code(&[c]), c as char);
        }

        assert_error!(parse_subfield_code(&[b'!']));

        Ok(())
    }

    #[test]
    fn test_parse_subfield_value() -> TestResult {
        assert_done_and_eq!(parse_subfield_value(b"abc"), "abc");
        assert_done_and_eq!(parse_subfield_value(b"a\x1ebc"), "a");
        assert_done_and_eq!(parse_subfield_value(b"a\x1fbc"), "a");
        assert_done_and_eq!(parse_subfield_value(b""), "");

        Ok(())
    }

    #[test]
    fn test_parse_subfield() -> TestResult {
        assert_done_and_eq!(
            parse_subfield(b"\x1fa123"),
            SubfieldRef {
                code: 'a',
                value: "123".into()
            }
        );

        assert_done_and_eq!(
            parse_subfield(b"\x1fa"),
            SubfieldRef {
                code: 'a',
                value: "".into()
            }
        );

        assert!(parse_subfield(b"a123").is_err());
        assert!(parse_subfield(b"").is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_ref_from_bytes() -> TestResult {
        assert_eq!(
            SubfieldRef::from_bytes(b"\x1f0123456789X\x1fabc")?,
            SubfieldRef {
                code: '0',
                value: b"123456789X".as_bstr()
            }
        );

        assert!(SubfieldRef::from_bytes(b"0123456789X\x1fabc").is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_from_bytes() -> TestResult {
        assert_eq!(
            Subfield::from_bytes(b"\x1f0123456789X\x1fabc")?,
            Subfield {
                code: '0',
                value: "123456789X".into(),
            }
        );

        assert!(Subfield::from_bytes(b"0123456789X\x1fabc").is_err());
        Ok(())
    }
    #[test]
    fn test_subfield_code() -> TestResult {
        let subfield = Subfield::from_bytes(b"\x1faabc")?;
        assert_eq!(subfield.code(), 'a');

        Ok(())
    }

    #[test]
    fn test_subfield_value() -> TestResult {
        let subfield = Subfield::from_bytes(b"\x1faabc")?;
        assert_eq!(subfield.value(), &BString::from("abc"));

        Ok(())
    }

    #[test]
    fn test_subfield_validate() -> TestResult {
        let subfield = Subfield::from_bytes(b"\x1f0123456789X")?;
        assert!(subfield.validate().is_ok());

        let subfield = Subfield::from_bytes(&[b'\x1f', b'0', 0, 157])?;
        assert!(subfield.validate().is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_write() -> TestResult {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield = Subfield::from_bytes(b"\x1f0123456789X")?;
        subfield.write(&mut writer)?;

        assert_eq!(String::from_utf8(writer.into_inner())?, "\x1f0123456789X");

        Ok(())
    }

    #[test]
    fn test_subfield_fmt() -> TestResult {
        let subfield = Subfield::from_str("\x1f0123456789X")?;
        assert_eq!(format!("{}", subfield), "$0123456789X");

        Ok(())
    }
}
