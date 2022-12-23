//! This module contains data structures and functions related to
//! PICA+ subfield.

use std::fmt;

use bstr::{BString, ByteSlice};
use nom::bytes::complete::is_not;
use nom::character::complete::{char, satisfy};
use nom::combinator::{cut, map, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::common::ParseResult;
use crate::error::{Error, Result};

/// A PICA+ subfield, that may contian invalid UTF-8 data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subfield {
    pub(crate) code: char,
    pub(crate) value: BString,
}

/// Parses a subfield code.
pub(crate) fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    map(satisfy(|c| c.is_ascii_alphanumeric()), char::from)(i)
}

/// Parses a subfield value.
fn parse_subfield_value(i: &[u8]) -> ParseResult<BString> {
    map(recognize(many0(is_not("\x1E\x1F"))), BString::from)(i)
}

/// Parses a subfield.
pub(crate) fn parse_subfield(i: &[u8]) -> ParseResult<Subfield> {
    map(
        preceded(
            char('\x1F'),
            cut(pair(parse_subfield_code, parse_subfield_value)),
        ),
        |(code, value)| Subfield::from_unchecked(code, value),
    )(i)
}

impl Subfield {
    /// Creates a new `Subfield`
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::error::Error;
    ///
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     assert!(Subfield::new('0', "12283643X").is_ok());
    ///     assert!(Subfield::new('!', "12283643X").is_err());
    ///     assert!(Subfield::new('a', "123\x1f34").is_err());
    ///     assert!(Subfield::new('a', "123\x1e34").is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S>(code: char, value: S) -> Result<Subfield>
    where
        S: Into<BString>,
    {
        if !code.is_ascii_alphanumeric() {
            return Err(Error::InvalidSubfield(format!(
                "Invalid subfield code '{code}'"
            )));
        }

        let value: BString = value.into();
        if value.contains(&b'\x1e') || value.contains(&b'\x1f') {
            return Err(Error::InvalidSubfield(
                "Invalid subfield value.".to_string(),
            ));
        }

        Ok(Subfield { code, value })
    }

    /// Creates a new `Subfield` without checking the input
    #[inline]
    pub(crate) fn from_unchecked<S>(code: char, value: S) -> Self
    where
        S: Into<BString>,
    {
        Self {
            code,
            value: value.into(),
        }
    }

    /// Get a reference to the subfield's code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "12283643X")?;
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
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "12283643X")?;
    ///     assert_eq!(subfield.value(), "12283643X");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &BString {
        &self.value
    }

    /// Returns `true` if the subfield value is valid UTF-8 byte
    /// sequence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "123456789X")?;
    ///     assert_eq!(subfield.validate().is_ok(), true);
    ///
    ///     let subfield = Subfield::new('0', vec![0, 159])?;
    ///     assert_eq!(subfield.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.value.is_ascii() {
            return Ok(());
        }

        if let Err(e) = std::str::from_utf8(&self.value) {
            return Err(Error::Utf8Error(e));
        }

        Ok(())
    }

    /// Write the subfield into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{PicaWriter, Subfield, WriterBuilder};
    /// use std::error::Error;
    /// use tempfile::Builder;
    /// # use std::fs::read_to_string;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut tempfile = Builder::new().tempfile()?;
    ///     # let path = tempfile.path().to_owned();
    ///
    ///     let subfield = Subfield::new('0', "123456789X")?;
    ///     let mut writer = WriterBuilder::new().from_writer(tempfile);
    ///     subfield.write(&mut writer)?;
    ///     writer.finish()?;
    ///
    ///     # let result = read_to_string(path)?;
    ///     # assert_eq!(result, String::from("\x1f0123456789X"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> crate::error::Result<()> {
        write!(writer, "\x1f{}{}", self.code, self.value)?;
        Ok(())
    }
}

impl fmt::Display for Subfield {
    /// Format the subfield in a human-readable format.
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
            state.serialize_field(
                "value",
                &self.value.to_str_unchecked(),
            )?;
        }
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_subfield_new() -> TestResult {
        assert_eq!(
            Subfield::new('0', "abc")?,
            Subfield {
                code: '0',
                value: BString::from("abc")
            }
        );

        assert!(Subfield::new('!', "abc").is_err());
        assert!(Subfield::new('0', "a\x1fc").is_err());
        assert!(Subfield::new('0', "a\x1ec").is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_from_unchecked() -> TestResult {
        assert_eq!(
            Subfield::from_unchecked('0', "abc"),
            Subfield {
                code: '0',
                value: BString::from("abc")
            }
        );

        Ok(())
    }

    #[test]
    fn test_subfield_code() -> TestResult {
        let subfield = Subfield::new('a', "abc")?;
        assert_eq!(subfield.code(), 'a');

        Ok(())
    }

    #[test]
    fn test_subfield_value() -> TestResult {
        let subfield = Subfield::new('a', "abc")?;
        assert_eq!(subfield.value(), &BString::from("abc"));

        Ok(())
    }

    #[test]
    fn test_subfield_validate() -> TestResult {
        let subfield = Subfield::new('0', "123456789X")?;
        assert!(subfield.validate().is_ok());

        let subfield = Subfield::new('0', vec![0, 157])?;
        assert!(subfield.validate().is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_write() -> TestResult {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield = Subfield::new('0', "123456789X")?;
        subfield.write(&mut writer)?;

        assert_eq!(
            String::from_utf8(writer.into_inner())?,
            "\x1f0123456789X"
        );

        Ok(())
    }

    #[test]
    fn test_subfield_fmt() -> TestResult {
        let subfield = Subfield::new('0', "123456789X")?;
        assert_eq!(format!("{subfield}"), "$0123456789X");

        Ok(())
    }

    #[test]
    fn test_parse_subfield_code() -> TestResult {
        assert_eq!(parse_subfield_code(b"0")?.1, '0');
        assert_eq!(parse_subfield_code(b"a")?.1, 'a');
        assert_eq!(parse_subfield_code(b"Z")?.1, 'Z');
        assert!(parse_subfield_code(b"!").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_subfield_value() -> TestResult {
        assert_eq!(parse_subfield_value(b"abc")?.1, "abc");
        assert_eq!(parse_subfield_value(b"a\x1ebc")?.1, "a");
        assert_eq!(parse_subfield_value(b"a\x1fbc")?.1, "a");
        assert_eq!(parse_subfield_value(b"")?.1, "");

        Ok(())
    }

    #[test]
    fn test_parse_subfield() -> TestResult {
        assert_eq!(
            parse_subfield(b"\x1fa123")?.1,
            Subfield::new('a', "123")?
        );
        assert_eq!(
            parse_subfield(b"\x1fa")?.1,
            Subfield::new('a', "")?
        );
        assert!(parse_subfield(b"a123").is_err());
        assert!(parse_subfield(b"").is_err());

        Ok(())
    }
}
