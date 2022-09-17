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

/// A immutable PICA+ subfield.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubfieldRef<'a>(pub(crate) char, pub(crate) &'a BStr);

impl<'a> SubfieldRef<'a> {
    /// Create a new subfield reference.
    ///
    /// # Panics
    ///
    /// This method panics if the subfield code or the value is invalid.
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
    ///     assert_eq!(subfield.value(), "bcd");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(code: char, value: impl Into<&'a BStr>) -> Self {
        let value = value.into();

        assert!(
            code.is_ascii_alphanumeric()
                && value.find_byte(b'\x1e').is_none()
                && value.find_byte(b'\x1f').is_none()
        );

        Self(code, value)
    }

    /// Creates an immutable PICA+ subfield from a byte slice.
    ///
    /// If an invalid subfield is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(SubfieldRef::from_bytes(b"\x1f0123456789X").is_ok());
    ///     assert!(SubfieldRef::from_bytes(b"abc").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_subfield_ref(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidSubfield)
            .map(|(_, subfield)| subfield)
    }

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
        self.0
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
    ///     let subfield = SubfieldRef::new('0', "0123456789X");
    ///     assert_eq!(subfield.value(), "0123456789X");
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &'a BStr {
        self.1
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
    ///     let subfield = SubfieldRef::new('0', "");
    ///     assert!(subfield.is_empty());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.1.len() == 0
    }

    /// Converts the immutable subfield into its mutable counterpart by
    /// consuming the source.
    pub fn into_owned(self) -> Subfield {
        self.into()
    }

    /// Converts the immutable subfield into its mutable counterpart.
    pub fn to_owned(&self) -> Subfield {
        self.clone().into()
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
        if self.1.is_ascii() {
            return Ok(());
        }

        std::str::from_utf8(self.1)?;
        Ok(())
    }

    /// Write the subfield into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let subfield = Subfield::new('0', "123456789X");
    ///     subfield.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #    String::from_utf8(writer.into_inner())?,
    ///     #    "\x1f0123456789X"
    ///     # );
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "\x1f{}{}", self.0, self.1)
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

/// Parse a PICA+ subfield reference.
pub(crate) fn parse_subfield_ref(i: &[u8]) -> ParseResult<SubfieldRef> {
    map(
        preceded(
            char('\x1f'),
            pair(parse_subfield_code, parse_subfield_value),
        ),
        |(code, value)| SubfieldRef(code, value),
    )(i)
}

/// A mutable PICA+ subfield.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subfield(pub(crate) char, pub(crate) BString);

impl Subfield {
    /// Create a new subfield.
    ///
    /// # Panics
    ///
    /// This method panics if the subfield code or the value is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('a', "bcd");
    ///     assert_eq!(subfield.code(), 'a');
    ///     assert_eq!(subfield.value(), "bcd");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(code: char, value: impl Into<BString>) -> Self {
        let value = value.into();

        assert!(
            code.is_ascii_alphanumeric()
                && value.find_byte(b'\x1e').is_none()
                && value.find_byte(b'\x1f').is_none()
        );

        Self(code, value)
    }

    /// Creates an immutable PICA+ subfield from a byte slice.
    ///
    /// If an invalid subfield is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(Subfield::from_bytes(b"\x1f0123456789X").is_ok());
    ///     assert!(Subfield::from_bytes(b"abc").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParsePicaError> {
        Ok(Self::from(SubfieldRef::from_bytes(data)?))
    }

    /// Returns the code of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "0123456789X");
    ///     assert_eq!(subfield.code(), '0');
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> char {
        self.0
    }

    /// Returns the value of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "0123456789X");
    ///     assert_eq!(subfield.value(), "0123456789X");
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &BString {
        &self.1
    }

    /// Returns true if the subfield value is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "");
    ///     assert!(subfield.is_empty());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.1.len() == 0
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the subfield
    /// value contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = Subfield::new('0', "123456789X");
    ///     assert_eq!(subfield.validate().is_ok(), true);
    ///
    ///     let subfield = Subfield::from_bytes(&[b'\x1f', b'0', 0, 159])?;
    ///     assert_eq!(subfield.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), Utf8Error> {
        if self.1.is_ascii() {
            return Ok(());
        }

        std::str::from_utf8(&self.1)?;
        Ok(())
    }

    /// Write the subfield into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let subfield = Subfield::new('0', "123456789X");
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
        write!(out, "\x1f{}{}", self.0, self.1)
    }
}

impl From<SubfieldRef<'_>> for Subfield {
    #[inline]
    fn from(subfield: SubfieldRef<'_>) -> Self {
        Self(subfield.0, subfield.1.into())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_subfield_ref_new() {
        let subfield = SubfieldRef::new('a', "abc");
        assert!(subfield.validate().is_ok());
        assert_eq!(subfield.code(), 'a');
        assert_eq!(subfield.value(), "abc");
        assert!(!subfield.is_empty());

        let subfield = SubfieldRef::new('a', "");
        assert!(subfield.is_empty());
    }

    #[test]
    fn test_subfield_ref_from_bytes() {
        let subfield =
            SubfieldRef::from_bytes(b"\x1f0123456789X").unwrap();
        assert!(subfield.validate().is_ok());
        assert_eq!(subfield.value(), "123456789X");
        assert_eq!(subfield.code(), '0');

        assert_eq!(
            SubfieldRef::from_bytes(b"\x1faabc").unwrap(),
            SubfieldRef::new('a', "abc")
        );

        assert_eq!(
            SubfieldRef::from_bytes(b"abc").unwrap_err(),
            ParsePicaError::InvalidSubfield,
        );
    }

    #[test]
    #[should_panic]
    fn test_subfield_ref_invalid_code() {
        SubfieldRef::new('!', "abc");
    }

    #[test]
    #[should_panic]
    fn test_subfield_ref_invalid_value1() {
        SubfieldRef::new('0', "\x1f");
    }

    #[test]
    #[should_panic]
    fn test_subfield_ref_invalid_value2() {
        SubfieldRef::new('0', "\x1e");
    }

    #[test]
    fn test_subfield_ref_write_to() -> anyhow::Result<()> {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield = SubfieldRef::new('0', "123456789X");
        subfield.write_to(&mut writer)?;

        assert_eq!(
            String::from_utf8(writer.into_inner())?,
            "\x1f0123456789X"
        );

        Ok(())
    }

    #[test]
    fn test_subfield_new() {
        let subfield = Subfield::new('a', "abc");
        assert_eq!(subfield.code(), 'a');
        assert_eq!(subfield.value(), "abc");
        assert!(!subfield.is_empty());

        let subfield = Subfield::new('a', "");
        assert!(subfield.is_empty());
    }

    #[test]
    fn test_subfield_from_bytes() {
        let subfield =
            Subfield::from_bytes(b"\x1f0123456789X").unwrap();
        assert_eq!(subfield.value(), "123456789X");
        assert_eq!(subfield.code(), '0');

        assert_eq!(
            Subfield::from_bytes(b"\x1faabc").unwrap(),
            Subfield::new('a', "abc")
        );

        assert_eq!(
            Subfield::from_bytes(b"abc").unwrap_err(),
            ParsePicaError::InvalidSubfield,
        );
    }

    #[test]
    #[should_panic]
    fn test_subfield_invalid_code() {
        Subfield::new('!', "abc");
    }

    #[test]
    #[should_panic]
    fn test_subfield_invalid_value1() {
        Subfield::new('0', "\x1f");
    }

    #[test]
    #[should_panic]
    fn test_subfield_invalid_value2() {
        Subfield::new('0', "\x1e");
    }

    #[test]
    fn test_subfield_write_to() -> anyhow::Result<()> {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield = Subfield::new('0', "123456789X");
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
            parse_subfield_ref(b"\x1fa123"),
            SubfieldRef('a', "123".into())
        );

        assert_done_and_eq!(
            parse_subfield_ref(b"\x1fa"),
            SubfieldRef('a', "".into())
        );

        assert!(parse_subfield_ref(b"a123").is_err());
        assert!(parse_subfield_ref(b"").is_err());
    }
}
