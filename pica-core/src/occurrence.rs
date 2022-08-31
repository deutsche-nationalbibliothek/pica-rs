//! This module contains data structures and functions related to PICA+
//! occurrences.

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use bstr::{BStr, BString, ByteSlice};
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::char;
use nom::character::is_digit;
use nom::sequence::preceded;
use nom::Finish;

use crate::{ParseError, ParseResult};

/// An immutable PICA+ occurrence.
#[derive(Debug, PartialEq, Eq)]
pub struct OccurrenceRef<'a>(&'a BStr);

/// Parse a PICA+ occurrence.
#[inline]
pub fn parse_occurrence<'a>(i: &'a [u8]) -> ParseResult<OccurrenceRef<'a>> {
    let (i, value) = preceded(char('/'), take_while_m_n(2, 3, is_digit))(i)?;

    Ok((i, OccurrenceRef(value.as_bstr())))
}

impl<'a> OccurrenceRef<'a> {
    /// Creates an immutable PICA+ occurrence from a byte slice.
    ///
    /// ```rust
    /// use pica_core::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(OccurrenceRef::from_bytes(b"/00").is_ok());
    ///     assert!(OccurrenceRef::from_bytes(b"/A").is_err());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(match parse_occurrence(data).finish() {
            Ok((_, tag)) => tag,
            _ => return Err(ParseError::InvalidOccurrence),
        })
    }
}

/// A mutable PICA+ occurrence.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct Occurrence(BString);

impl Occurrence {
    /// Creates an immutable PICA+ occurrence from a byte slice.
    ///
    /// ```rust
    /// use pica_core::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(Occurrence::from_bytes(b"/00").is_ok());
    ///     assert!(Occurrence::from_bytes(b"/A").is_err());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        Ok(OccurrenceRef::from_bytes(data)?.into())
    }

    #[inline]
    pub fn from_unchecked<S: Into<BString>>(occurrence: S) -> Self {
        Self(occurrence.into())
    }
}

impl From<OccurrenceRef<'_>> for Occurrence {
    fn from(occ_ref: OccurrenceRef<'_>) -> Self {
        Occurrence(occ_ref.0.into())
    }
}

impl Deref for Occurrence {
    type Target = BString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<str> for Occurrence {
    fn eq(&self, other: &str) -> bool {
        self.0 == other.as_bytes()
    }
}

impl FromStr for Occurrence {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Occurrence::from_bytes(s.as_bytes())
    }
}

impl fmt::Display for Occurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TestResult;

    use nom_test_helpers::prelude::*;

    #[test]
    fn test_parse_occurrence() -> TestResult {
        assert_done_and_eq!(
            parse_occurrence(b"/000"),
            OccurrenceRef(b"000".as_bstr())
        );
        assert_done_and_eq!(
            parse_occurrence(b"/001"),
            OccurrenceRef(b"001".as_bstr())
        );
        assert_done_and_eq!(
            parse_occurrence(b"/00"),
            OccurrenceRef(b"00".as_bstr())
        );

        assert_error!(parse_occurrence(b"//00"));
        assert_error!(parse_occurrence(b"00"));

        Ok(())
    }

    #[test]
    fn test_occurrence_ref_from_bytes() -> TestResult {
        assert_eq!(
            OccurrenceRef::from_bytes(b"/01")?,
            OccurrenceRef("01".into())
        );

        assert_eq!(
            OccurrenceRef::from_bytes(b"01").unwrap_err().to_string(),
            "parse error: invalid occurrence"
        );

        assert!(OccurrenceRef::from_bytes(b"/0A").is_err());

        Ok(())
    }

    #[test]
    fn test_occurrence_from_occurrence_ref() -> TestResult {
        assert_eq!(
            Occurrence::from(OccurrenceRef::from_bytes(b"/04")?),
            Occurrence("04".into())
        );

        Ok(())
    }

    #[test]
    fn test_occurrence_from_str() -> TestResult {
        assert_eq!(Occurrence::from_str("/01")?, Occurrence("01".into()));
        assert!(Occurrence::from_str("/0A").is_err());

        Ok(())
    }

    #[test]
    fn test_occurrence_to_string() -> TestResult {
        assert_eq!(Occurrence::from_str("/01")?.to_string(), "/01".to_string());
        Ok(())
    }
}
