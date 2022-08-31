//! This module contains data structures and functions related to
//! PICA+ occurrences.

use std::fmt;
use std::ops::Deref;

use bstr::BString;

use nom::character::complete::{char, satisfy};
use nom::combinator::{all_consuming, cut, map, recognize};
use nom::multi::many_m_n;
use nom::sequence::preceded;
use nom::Finish;

use crate::common::ParseResult;
use crate::error::Error;

/// A PICA+ occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Occurrence(BString);

/// Parses digits of a PICA+ occurrence.
#[inline]
pub(crate) fn parse_occurrence_digits(i: &[u8]) -> ParseResult<&[u8]> {
    recognize(many_m_n(2, 3, satisfy(|c| c.is_ascii_digit())))(i)
}

/// Parses a PICA+ occurrence.
#[inline]
pub(crate) fn parse_occurrence(i: &[u8]) -> ParseResult<Occurrence> {
    map(
        preceded(char('/'), cut(recognize(parse_occurrence_digits))),
        Occurrence::from_unchecked,
    )(i)
}

impl Occurrence {
    /// Creates a PICA+ occurrence from a string slice.
    ///
    /// If an invalid occurrence is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(Occurrence::new("00").is_ok());
    ///     assert!(Occurrence::new("/00").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        let mut parser =
            map(all_consuming(parse_occurrence_digits), Self::from_unchecked);

        match parser(data.as_ref().as_bytes()).finish() {
            Ok((_, occurrence)) => Ok(occurrence),
            Err(_) => {
                Err(Error::InvalidOccurrence("Invalid occurrence".to_string()))
            }
        }
    }

    /// Creates a new `Occurrence` without checking the input
    pub(crate) fn from_unchecked<S: Into<BString>>(occurrence: S) -> Self {
        Self(occurrence.into())
    }
}

impl fmt::Display for Occurrence {
    /// Format the occurrence in a human-readable format.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
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

#[cfg(test)]
impl quickcheck::Arbitrary for Occurrence {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Occurrence::from_unchecked(vec![
            *g.choose(b"0123456789").unwrap(),
            *g.choose(b"0123456789").unwrap(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_occurrence_digits() -> TestResult {
        assert_eq!(parse_occurrence_digits(b"00")?.1, b"00");
        assert_eq!(parse_occurrence_digits(b"01")?.1, b"01");
        assert_eq!(parse_occurrence_digits(b"99")?.1, b"99");
        assert_eq!(parse_occurrence_digits(b"000")?.1, b"000");
        assert_eq!(parse_occurrence_digits(b"001")?.1, b"001");
        assert_eq!(parse_occurrence_digits(b"99")?.1, b"99");
        assert!(parse_occurrence_digits(b"/00").is_err());
        assert!(parse_occurrence_digits(b"a0").is_err());
        assert!(parse_occurrence_digits(b"0a").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_occurrence() -> TestResult {
        assert_eq!(
            parse_occurrence(b"/00")?.1,
            Occurrence(BString::from("00"))
        );
        assert_eq!(
            parse_occurrence(b"/001")?.1,
            Occurrence(BString::from("001"))
        );

        assert!(parse_occurrence(b"//00").is_err());
        assert!(parse_occurrence(b"00").is_err());

        Ok(())
    }

    #[test]
    fn test_occurrence_new() -> TestResult {
        assert_eq!(Occurrence::new("00")?, Occurrence(BString::from("00")));
        assert_eq!(Occurrence::new("001")?, Occurrence(BString::from("001")));
        assert!(Occurrence::new("/00").is_err());
        assert!(Occurrence::new("a0").is_err());
        assert!(Occurrence::new("0a").is_err());

        Ok(())
    }

    #[test]
    fn test_occurrence_from_unchecked() -> TestResult {
        assert_eq!(
            Occurrence::from_unchecked("00"),
            Occurrence(BString::from("00"))
        );

        Ok(())
    }
}
