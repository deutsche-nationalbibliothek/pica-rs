//! This module contains data structures and functions related to
//! PICA+ occurrences.

use crate::parser::ParseResult;

use std::cmp::PartialEq;
use std::fmt;
use std::ops::Deref;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, satisfy};
use nom::combinator::{cut, map, recognize, success, verify};
use nom::multi::many_m_n;
use nom::sequence::{preceded, separated_pair};

use bstr::BString;

/// A PICA+ occurrence.
#[derive(Debug, PartialEq, Clone)]
pub struct Occurrence(pub(crate) BString);

#[derive(Debug)]
pub struct ParseOccurrenceError(pub(crate) String);

impl std::error::Error for ParseOccurrenceError {}

impl fmt::Display for ParseOccurrenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn parse_occurrence_digits(i: &[u8]) -> ParseResult<&[u8]> {
    recognize(many_m_n(2, 3, satisfy(|c| c.is_ascii_digit())))(i)
}

impl Occurrence {
    /// Creates a PICA+ occurrence from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(Occurrence::new("00").is_ok());
    ///     assert!(Occurrence::new("01").is_ok());
    ///     assert!(Occurrence::new("100").is_ok());
    ///     assert!(Occurrence::new("0a").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T>(occurrence: T) -> Result<Self, ParseOccurrenceError>
    where
        T: Into<BString>,
    {
        let occurrence = occurrence.into();

        if occurrence.len() < 2 || occurrence.len() > 3 {
            return Err(ParseOccurrenceError(
                "length < 2 || length > 3".to_string(),
            ));
        }

        if !occurrence.iter().all(u8::is_ascii_digit) {
            return Err(ParseOccurrenceError(format!(
                "Invalid occurrence '{}'",
                occurrence
            )));
        }

        Ok(Self(occurrence))
    }

    /// Parse a PICA+ occurrence.
    ///
    /// # Grammar
    ///
    /// ```ebnf
    /// occurrence ::= '/' [0-9]{2,3}
    /// ```
    #[inline]
    pub(crate) fn parse_occurrence(i: &[u8]) -> ParseResult<Self> {
        map(
            preceded(char('/'), cut(recognize(parse_occurrence_digits))),
            Occurrence::from_unchecked,
        )(i)
    }

    /// Creates a new `Occurrence` without checking the input.
    #[inline]
    pub(crate) fn from_unchecked<T>(occurrence: T) -> Self
    where
        T: Into<BString>,
    {
        Self(occurrence.into())
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

impl fmt::Display for Occurrence {
    /// Format the field in a human-readable format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, Occurrence};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let occurrence = Occurrence::new("01")?;
    ///     assert_eq!(format!("{}", occurrence), "/01");
    ///
    ///     Ok(())
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.0)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OccurrenceMatcher {
    Some(Occurrence),
    Range(Occurrence, Occurrence),
    Any,
    None,
}

#[derive(Debug)]
pub enum ParseOccurrenceMatcherError {
    InvalidOccurrence(ParseOccurrenceError),
    InvalidRange(String),
}

impl std::error::Error for ParseOccurrenceMatcherError {}

impl fmt::Display for ParseOccurrenceMatcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidOccurrence(ref e) => e.fmt(f),
            Self::InvalidRange(ref m) => f.write_str(m),
        }
    }
}

impl From<ParseOccurrenceError> for ParseOccurrenceMatcherError {
    fn from(err: ParseOccurrenceError) -> Self {
        Self::InvalidOccurrence(err)
    }
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
    pub fn new<T>(value: T) -> Result<Self, ParseOccurrenceMatcherError>
    where
        T: Into<BString>,
    {
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
    pub fn range<T>(min: T, max: T) -> Result<Self, ParseOccurrenceMatcherError>
    where
        T: Into<BString> + PartialOrd,
    {
        if min >= max {
            return Err(ParseOccurrenceMatcherError::InvalidRange(
                "min >= max".to_string(),
            ));
        }

        Ok(OccurrenceMatcher::Range(
            Occurrence::new(min)?,
            Occurrence::new(max)?,
        ))
    }

    /// Parses a occurrence matcher
    #[inline]
    pub(crate) fn parse_occurrence_matcher(input: &[u8]) -> ParseResult<Self> {
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
                        |x| {
                            OccurrenceMatcher::Some(Occurrence::from_unchecked(
                                x,
                            ))
                        },
                    ),
                    map(tag("00"), |_| OccurrenceMatcher::None),
                    map(char('*'), |_| OccurrenceMatcher::Any),
                ))),
            ),
            success(OccurrenceMatcher::None),
        ))(input)
    }
}

impl PartialEq<OccurrenceMatcher> for Option<Occurrence> {
    /// Equality comparision between `OccurrenceMatcher` and an
    /// `Option<Occurrence>`
    ///
    /// ```rust
    /// use pica::{Occurrence, OccurrenceMatcher};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(Some(Occurrence::new("001")?) == OccurrenceMatcher::Any);
    ///     assert!(None == OccurrenceMatcher::Any);
    ///
    ///     Ok(())
    /// }
    /// ```
    fn eq(&self, other: &OccurrenceMatcher) -> bool {
        match other {
            OccurrenceMatcher::Any => true,
            OccurrenceMatcher::None => {
                if let Some(ref rhs) = self {
                    rhs == "00"
                } else {
                    true
                }
            }
            OccurrenceMatcher::Some(lhs) => {
                if let Some(rhs) = self {
                    lhs == rhs
                } else {
                    false
                }
            }
            OccurrenceMatcher::Range(min, max) => {
                if let Some(rhs) = self {
                    (rhs.0 >= min.0) && (rhs.0 <= max.0)
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_occurrence() {
        assert_eq!(
            Occurrence::parse_occurrence(b"/00").unwrap().1,
            Occurrence::new("00").unwrap()
        );

        assert_eq!(
            Occurrence::parse_occurrence(b"/01").unwrap().1,
            Occurrence::new("01").unwrap()
        );

        assert_eq!(
            Occurrence::parse_occurrence(b"/001").unwrap().1,
            Occurrence::new("001").unwrap()
        );

        assert!(Occurrence::parse_occurrence(b"/XYZ").is_err());
    }

    #[test]
    fn test_parse_occurrence_matcher() {
        // Some
        assert_eq!(
            OccurrenceMatcher::parse_occurrence_matcher(b"/01")
                .unwrap()
                .1,
            OccurrenceMatcher::new("01").unwrap()
        );
        assert_eq!(
            OccurrenceMatcher::parse_occurrence_matcher(b"/001")
                .unwrap()
                .1,
            OccurrenceMatcher::new("001").unwrap()
        );
        // Any
        assert_eq!(
            OccurrenceMatcher::parse_occurrence_matcher(b"/*")
                .unwrap()
                .1,
            OccurrenceMatcher::Any
        );
        // None
        assert_eq!(
            OccurrenceMatcher::parse_occurrence_matcher(b"/00")
                .unwrap()
                .1,
            OccurrenceMatcher::None
        );
        assert_eq!(
            OccurrenceMatcher::parse_occurrence_matcher(b"abc")
                .unwrap()
                .1,
            OccurrenceMatcher::None
        );
    }
}
