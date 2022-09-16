use std::io::{self, Write};
use std::ops::Deref;

use bstr::{BStr, BString, ByteSlice};
use nom::character::complete::{char, satisfy};
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::sequence::{preceded, tuple};
use nom::Finish;

use crate::parser::ParseResult;
use crate::ParsePicaError;

/// A immutable PICA+ occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccurrenceRef<'a>(pub(crate) &'a BStr);

impl<'a> OccurrenceRef<'a> {
    /// Creates an immutable PICA+ occurrence from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = OccurrenceRef::new("01");
    ///     assert_eq!(occurrence, "01");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(digits: impl Into<&'a BStr>) -> Self {
        let result =
            all_consuming(parse_occurrence_digits)(digits.into())
                .finish()
                .unwrap();

        Self(result.1)
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
        all_consuming(parse_occurrence_ref)(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidOccurrence)
            .map(|(_, occurrence)| occurrence)
    }

    /// Converts the immutable subfield into its mutable counterpart by
    /// consuming the source.
    pub fn into_owned(self) -> Occurrence {
        self.into()
    }

    /// Converts the immutable subfield into its mutable counterpart.
    pub fn to_owned(&self) -> Occurrence {
        self.clone().into()
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

impl PartialEq<&str> for OccurrenceRef<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<str> for OccurrenceRef<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0 == other
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
pub fn parse_occurrence_ref(i: &[u8]) -> ParseResult<OccurrenceRef> {
    map(preceded(char('/'), parse_occurrence_digits), |digits| {
        OccurrenceRef(digits)
    })(i)
}

/// A mutable PICA+ occurrence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence(pub(crate) BString);

impl Occurrence {
    /// Creates an mutable PICA+ occurrence from a byte slice.
    ///
    /// If an invalid tag is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let occurrence = Occurrence::new("01");
    ///     assert_eq!(occurrence, "01");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(digits: impl Into<BString>) -> Self {
        OccurrenceRef::new(digits.into().as_bstr()).into_owned()
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

impl Deref for Occurrence {
    type Target = BString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<OccurrenceRef<'_>> for Occurrence {
    #[inline]
    fn from(occurrence: OccurrenceRef<'_>) -> Self {
        Self(occurrence.0.into())
    }
}

impl PartialEq<&str> for Occurrence {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<str> for Occurrence {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0 == other
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
                parse_occurrence_ref(occurrence.as_bytes()),
                OccurrenceRef(occurrence[1..].into())
            )
        }

        for occurrence in ["00", "/0A", "/!0", "/9x"] {
            assert_error!(parse_occurrence_ref(occurrence.as_bytes()))
        }
    }

    #[quickcheck]
    fn test_parse_arbitrary_occurrence(occurrence: Occurrence) -> bool {
        let mut value = String::from("/");
        value.push_str(&occurrence.to_string());

        OccurrenceRef::from_bytes(value.as_bytes()).is_ok()
    }
}
