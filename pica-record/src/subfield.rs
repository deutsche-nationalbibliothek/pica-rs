use bstr::{BStr, ByteSlice};
use nom::bytes::complete::take_till;
use nom::character::complete::satisfy;
use nom::combinator::map;

use crate::parser::{ParseResult, RS, US};

/// An immutable PICA+ subfield.
#[derive(Debug, PartialEq, Eq)]
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
}

/// Parse a PICA+ subfield code.
pub fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

/// Parse a PICA+ subfield value.
pub fn parse_subfield_value(i: &[u8]) -> ParseResult<&BStr> {
    map(take_till(|c| c == US || c == RS), ByteSlice::as_bstr)(i)
}

#[cfg(test)]
mod tests {

    use nom_test_helpers::prelude::*;

    use super::*;

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
}
