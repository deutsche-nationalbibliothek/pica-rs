use std::fmt::Display;
use std::io::{self, Write};

use bstr::{BStr, BString, ByteSlice};
use nom::character::complete::{char, satisfy};
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::sequence::{preceded, tuple};
use nom::Finish;

use crate::parser::ParseResult;
use crate::ParsePicaError;

/// A PICA+ occurrence.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Occurrence<T>(pub(crate) T);

/// A immutable PICA+ occurrence.
pub type OccurrenceRef<'a> = Occurrence<&'a BStr>;

/// A mutable PICA+ occurrence.
pub type OccurrenceMut = Occurrence<BString>;

impl<'a, T: AsRef<[u8]> + From<&'a BStr> + Display> Occurrence<T> {
    /// Create a new PICA+ occurrence.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = OccurrenceRef::new("01");
    ///     assert_eq!(occurrence, "01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(value: impl Into<T>) -> Self {
        let value = value.into();

        all_consuming(parse_occurrence_digits)(value.as_ref())
            .finish()
            .map_err(|_| ParsePicaError::InvalidOccurrence)
            .unwrap();

        Self(value)
    }

    /// Creates an immutable PICA+ tag from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(OccurrenceRef::from_bytes(b"/01").is_ok());
    ///     assert!(OccurrenceRef::from_bytes(b"01").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParsePicaError> {
        all_consuming(parse_occurrence)(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidOccurrence)
            .map(|(_, digits)| Occurrence(digits.into()))
    }

    /// Write the occurrence into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let occurrence = OccurrenceRef::new("01");
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

/// Parse the digits of an PICA+ occurrence.
#[inline]
pub fn parse_occurrence_digits(i: &[u8]) -> ParseResult<&BStr> {
    map(
        recognize(tuple((
            satisfy(|c| c.is_ascii_digit()),
            satisfy(|c| c.is_ascii_digit()),
            opt(satisfy(|c| c.is_ascii_digit())),
        ))),
        ByteSlice::as_bstr,
    )(i)
}

/// Parse a PICA+ occurrence (read-only).
pub fn parse_occurrence(i: &[u8]) -> ParseResult<&BStr> {
    preceded(char('/'), parse_occurrence_digits)(i)
}

impl<T: AsRef<[u8]>> PartialEq<&str> for Occurrence<T> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == other.as_bytes()
    }
}

impl<T: AsRef<[u8]>> PartialEq<str> for Occurrence<T> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self == other
    }
}

impl<'a> From<OccurrenceRef<'a>> for OccurrenceMut {
    #[inline]
    fn from(occurrence: Occurrence<&'a BStr>) -> Self {
        Self(occurrence.0.into())
    }
}

impl<'a> OccurrenceRef<'a> {
    /// Converts the immutable occurrence into its mutable counterpart
    /// by consuming the source.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = OccurrenceRef::new("01").into_owned();
    ///     assert_eq!(occurrence, "01");
    ///     Ok(())
    /// }
    /// ```
    pub fn into_owned(self) -> OccurrenceMut {
        self.into()
    }

    /// Converts the immutable tag into its mutable counterpart.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = OccurrenceRef::new("01").to_owned();
    ///     assert_eq!(occurrence, "01");
    ///     Ok(())
    /// }
    /// ```
    pub fn to_owned(&self) -> OccurrenceMut {
        self.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_parse_occurrence_ref() {
        for occurrence in ["/00", "/01", "/000", "/123"] {
            assert_done_and_eq!(
                parse_occurrence(occurrence.as_bytes()),
                occurrence[1..].as_bytes()
            )
        }

        for occurrence in ["00", "/0A", "/!0", "/9x"] {
            assert_error!(parse_occurrence(occurrence.as_bytes()))
        }
    }
}
