//! This module contains data structures and functions related to
//! PICA+ occurrences.

use std::fmt;
use std::ops::Deref;

use bstr::BString;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, satisfy};
use nom::combinator::{all_consuming, cut, map, recognize, success, verify};
use nom::multi::many_m_n;
use nom::sequence::{preceded, separated_pair};
use nom::Finish;

use crate::common::ParseResult;
use crate::error::Error;

/// A PICA+ occurrence.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Occurrence(BString);

/// Parses digits of a PICA+ occurrence.
#[inline]
fn parse_occurrence_digits(i: &[u8]) -> ParseResult<&[u8]> {
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
        write!(f, "/{}", self.0)?;
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

#[derive(Debug, PartialEq, Clone)]
pub enum OccurrenceMatcher {
    Some(Occurrence),
    Range(Occurrence, Occurrence),
    Any,
    None,
}

impl OccurrenceMatcher {
    /// Creates a `OccurrenceMatcher::Some` variant.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Occurrence, OccurrenceMatcher};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = OccurrenceMatcher::new("001")?;
    ///     assert_eq!(matcher, OccurrenceMatcher::Some(Occurrence::new("001")?));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(value: S) -> Result<Self, Error> {
        Ok(OccurrenceMatcher::Some(Occurrence::new(value)?))
    }

    /// Creates a `OccurrenceMatcher::Occurrence`
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Occurrence, OccurrenceMatcher};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = OccurrenceMatcher::range("01", "03")?;
    ///     assert_eq!(
    ///         matcher,
    ///         OccurrenceMatcher::Range(
    ///             Occurrence::new("01")?,
    ///             Occurrence::new("03")?
    ///         )
    ///     );
    ///
    ///     assert!(OccurrenceMatcher::range("01", "01").is_err());
    ///     assert!(OccurrenceMatcher::range("03", "01").is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn range<T>(min: T, max: T) -> Result<Self, Error>
    where
        T: AsRef<str> + PartialOrd,
    {
        if min >= max {
            return Err(Error::InvalidOccurrenceMatcher(
                "Invalid OccurrenceMatcher: min >= max.".to_string(),
            ));
        }

        Ok(OccurrenceMatcher::Range(
            Occurrence::new(min)?,
            Occurrence::new(max)?,
        ))
    }
}

/// Parses a `OccurrenceMatcher`
#[inline]
pub(crate) fn parse_occurrence_matcher(
    i: &[u8],
) -> ParseResult<OccurrenceMatcher> {
    alt((
        preceded(
            char('/'),
            cut(alt((
                map(
                    verify(
                        separated_pair(
                            parse_occurrence_digits,
                            char('-'),
                            parse_occurrence_digits,
                        ),
                        |(min, max)| min < max,
                    ),
                    |(min, max)| {
                        OccurrenceMatcher::Range(
                            Occurrence::from_unchecked(min),
                            Occurrence::from_unchecked(max),
                        )
                    },
                ),
                map(
                    verify(parse_occurrence_digits, |x: &[u8]| x != b"00"),
                    |x| OccurrenceMatcher::Some(Occurrence::from_unchecked(x)),
                ),
                map(tag("00"), |_| OccurrenceMatcher::None),
                map(char('*'), |_| OccurrenceMatcher::Any),
            ))),
        ),
        success(OccurrenceMatcher::None),
    ))(i)
}

impl PartialEq<OccurrenceMatcher> for Occurrence {
    /// Equality comparision between `OccurrenceMatcher` and an
    /// `Occurrence`
    fn eq(&self, matcher: &OccurrenceMatcher) -> bool {
        match matcher {
            OccurrenceMatcher::Any => true,
            OccurrenceMatcher::None => self == "00",
            OccurrenceMatcher::Some(rhs) => self == rhs,
            OccurrenceMatcher::Range(min, max) => {
                (self >= min) && (self <= max)
            }
        }
    }
}

impl PartialEq<OccurrenceMatcher> for Option<Occurrence> {
    /// Equality comparision between `OccurrenceMatcher` and an
    /// `Option<Occurrence>`
    fn eq(&self, rhs: &OccurrenceMatcher) -> bool {
        match self {
            Some(occurrence) => occurrence == rhs,
            None => {
                matches!(rhs, OccurrenceMatcher::Any | OccurrenceMatcher::None)
            }
        }
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

    #[test]
    fn test_parse_occurrence_matcher() -> TestResult {
        assert_eq!(
            parse_occurrence_matcher(b"/01")?.1,
            OccurrenceMatcher::Some(Occurrence::new("01")?)
        );
        assert_eq!(
            parse_occurrence_matcher(b"/000")?.1,
            OccurrenceMatcher::Some(Occurrence::new("000")?)
        );

        // Range
        assert_eq!(
            parse_occurrence_matcher(b"/00-09")?.1,
            OccurrenceMatcher::range("00", "09")?
        );

        // None
        assert_eq!(
            parse_occurrence_matcher(b"/00")?.1,
            OccurrenceMatcher::None
        );
        assert_eq!(
            parse_occurrence_matcher(b"abc")?.1,
            OccurrenceMatcher::None
        );

        // Any
        assert_eq!(parse_occurrence_matcher(b"/*")?.1, OccurrenceMatcher::Any);

        Ok(())
    }

    #[test]
    fn test_occurrence_matcher_new() -> TestResult {
        assert_eq!(
            OccurrenceMatcher::new("00")?,
            OccurrenceMatcher::Some(Occurrence::new("00")?)
        );

        assert!(OccurrenceMatcher::new("abcdef").is_err());

        Ok(())
    }

    #[test]
    fn test_occurrence_matcher_range() -> TestResult {
        assert_eq!(
            OccurrenceMatcher::range("00", "03")?,
            OccurrenceMatcher::Range(
                Occurrence::new("00")?,
                Occurrence::new("03")?
            )
        );

        assert!(OccurrenceMatcher::range("03", "01").is_err());

        Ok(())
    }

    #[test]
    fn test_occurrence_eq_matcher() -> TestResult {
        assert_eq!(Occurrence::new("00")?, OccurrenceMatcher::new("00")?);
        assert_eq!(Occurrence::new("01")?, OccurrenceMatcher::new("01")?);
        assert_eq!(Occurrence::new("100")?, OccurrenceMatcher::new("100")?);
        assert_eq!(Occurrence::new("00")?, OccurrenceMatcher::None);
        assert_eq!(Occurrence::new("01")?, OccurrenceMatcher::Any);
        assert_eq!(
            Occurrence::new("01")?,
            OccurrenceMatcher::range("00", "09")?,
        );
        assert_ne!(
            Occurrence::new("01")?,
            OccurrenceMatcher::range("02", "09")?,
        );

        Ok(())
    }

    #[test]
    fn test_option_occurrence_eq_matcher() -> TestResult {
        // None
        assert_eq!(None, OccurrenceMatcher::None);
        assert_eq!(None, OccurrenceMatcher::Any);

        // Some
        assert_eq!(Some(Occurrence::new("00")?), OccurrenceMatcher::new("00")?);
        assert_eq!(Some(Occurrence::new("01")?), OccurrenceMatcher::new("01")?);
        assert_eq!(
            Some(Occurrence::new("100")?),
            OccurrenceMatcher::new("100")?
        );
        assert_eq!(Some(Occurrence::new("00")?), OccurrenceMatcher::None);
        assert_eq!(Some(Occurrence::new("01")?), OccurrenceMatcher::Any);
        assert_eq!(
            Some(Occurrence::new("01")?),
            OccurrenceMatcher::range("00", "09")?,
        );
        assert_ne!(
            Some(Occurrence::new("01")?),
            OccurrenceMatcher::range("02", "09")?,
        );

        Ok(())
    }
}
