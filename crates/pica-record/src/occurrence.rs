use std::io::{self, Write};

use bstr::{BStr, ByteSlice};
use winnow::combinator::preceded;
use winnow::stream::AsChar;
use winnow::token::take_while;
use winnow::{PResult, Parser};

use crate::ParsePicaError;

/// An immutable PICA+ occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Occurrence<'a>(&'a BStr);

/// Parse the digits of an PICA+ occurrence.
#[inline]
fn parse_occurrence_digits<'a>(i: &mut &'a [u8]) -> PResult<&'a BStr> {
    take_while(2..=3, AsChar::is_dec_digit)
        .map(ByteSlice::as_bstr)
        .parse_next(i)
}

/// Parse a PICA+ occurrence (read-only).
#[inline]
pub(crate) fn parse_occurrence<'a>(
    i: &mut &'a [u8],
) -> PResult<Occurrence<'a>> {
    preceded(b'/', parse_occurrence_digits)
        .map(|value| Occurrence(value.as_bstr()))
        .parse_next(i)
}

impl<'a> Occurrence<'a> {
    /// Create a new PICA+ occurrence.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = Occurrence::new("01");
    ///     assert_eq!(occurrence, "01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<B: ?Sized + AsRef<[u8]>>(value: &'a B) -> Self {
        Self::try_from(value.as_ref().as_bstr())
            .expect("value occurrence")
    }

    /// Creates an immutable PICA+ tag from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(Occurrence::from_bytes(b"/01").is_ok());
    ///     assert!(Occurrence::from_bytes(b"01").is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_occurrence
            .parse(bytes)
            .map_err(|_| ParsePicaError::InvalidOccurrence)
    }

    /// Write the occurrence into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let occurrence = Occurrence::new("01");
    ///     occurrence.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #    String::from_utf8(writer.into_inner())?,
    ///     #    "/01"
    ///     # );
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "/{}", self.0)
    }
}

impl<'a, T: AsRef<[u8]>> PartialEq<T> for Occurrence<'a> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl<'a> ToString for Occurrence<'a> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<'a> TryFrom<&'a BStr> for Occurrence<'a> {
    type Error = ParsePicaError;

    fn try_from(value: &'a BStr) -> Result<Self, Self::Error> {
        if parse_occurrence_digits.parse(value).is_err() {
            return Err(ParsePicaError::InvalidOccurrence);
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_occurrence_digits() {
        assert_eq!(
            parse_occurrence_digits.parse_peek(b"00").unwrap(),
            (b"".as_bytes(), b"00".as_bstr())
        );

        assert_eq!(
            parse_occurrence_digits.parse_peek(b"01").unwrap(),
            (b"".as_bytes(), b"01".as_bstr())
        );

        assert_eq!(
            parse_occurrence_digits.parse_peek(b"001").unwrap(),
            (b"".as_bytes(), b"001".as_bstr())
        );

        assert!(parse_occurrence_digits.parse_peek(b"0a").is_err());
        assert!(parse_occurrence_digits.parse_peek(b"").is_err());
        assert!(parse_occurrence_digits.parse_peek(b"0").is_err());
    }

    #[test]
    fn test_parse_occurrence() {
        assert_eq!(
            parse_occurrence.parse_peek(b"/01").unwrap(),
            (b"".as_bytes(), Occurrence(b"01".as_bstr()))
        );
    }
}
