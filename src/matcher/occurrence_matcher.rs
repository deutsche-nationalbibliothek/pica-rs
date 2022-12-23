use std::fmt;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{
    all_consuming, cut, map, success, value, verify,
};
use nom::sequence::{preceded, separated_pair};
use nom::Finish;

use crate::common::ParseResult;
use crate::occurrence::{parse_occurrence_digits, Occurrence};
use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OccurrenceMatcher {
    Some(Occurrence),
    Range(Occurrence, Occurrence),
    Any,
    None,
}

impl fmt::Display for OccurrenceMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Some(o) => write!(f, "/{o}"),
            Self::Range(from, to) => write!(f, "/{from}-{to}"),
            Self::Any => write!(f, "/*"),
            Self::None => write!(f, ""),
        }
    }
}

impl OccurrenceMatcher {
    /// Creates a occurrence matcher from a string slice.
    ///
    /// If an invalid occurrence matcher is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::OccurrenceMatcher;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(OccurrenceMatcher::new("/01").is_ok());
    ///     assert!(OccurrenceMatcher::new("/01-09").is_ok());
    ///     assert!(OccurrenceMatcher::new("/*").is_ok());
    ///     assert!(OccurrenceMatcher::new("").is_ok());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        let data = data.as_ref();

        match all_consuming(parse_occurrence_matcher)(data.as_bytes())
            .finish()
        {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(Error::InvalidMatcher(format!(
                "Expected valid occurrence matcher, got '{data}'"
            ))),
        }
    }

    /// Returns true, if and only if the given value matches against
    /// the occurrence matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::OccurrenceMatcher;
    /// use pica::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = OccurrenceMatcher::new("/01-09")?;
    ///     assert!(matcher.is_match(Some(&Occurrence::new("03")?)));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, occurrence: Option<&Occurrence>) -> bool {
        match occurrence {
            Some(occurrence) => match self {
                OccurrenceMatcher::Any => true,
                OccurrenceMatcher::None => occurrence == "00",
                OccurrenceMatcher::Some(rhs) => occurrence == rhs,
                OccurrenceMatcher::Range(min, max) => {
                    (occurrence >= min) && (occurrence <= max)
                }
            },
            None => {
                matches!(
                    self,
                    OccurrenceMatcher::Any | OccurrenceMatcher::None
                )
            }
        }
    }
}

impl From<Occurrence> for OccurrenceMatcher {
    fn from(occurrence: Occurrence) -> Self {
        Self::Some(occurrence)
    }
}

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
                        |(min, max)| {
                            min.len() == max.len() && min < max
                        },
                    ),
                    |(min, max)| {
                        OccurrenceMatcher::Range(
                            Occurrence::from_unchecked(min),
                            Occurrence::from_unchecked(max),
                        )
                    },
                ),
                map(
                    verify(parse_occurrence_digits, |x: &[u8]| {
                        x != b"00"
                    }),
                    |x| {
                        OccurrenceMatcher::Some(
                            Occurrence::from_unchecked(x),
                        )
                    },
                ),
                value(OccurrenceMatcher::None, tag("00")),
                value(OccurrenceMatcher::Any, char('*')),
            ))),
        ),
        success(OccurrenceMatcher::None),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_occurrence_matcher() -> TestResult {
        // Some
        let matcher = OccurrenceMatcher::new("/03")?;
        assert!(!matcher.is_match(Some(&Occurrence::new("00")?)));
        assert!(!matcher.is_match(Some(&Occurrence::new("02")?)));
        assert!(matcher.is_match(Some(&Occurrence::new("03")?)));
        assert!(!matcher.is_match(Some(&Occurrence::new("04")?)));

        // Range
        let matcher = OccurrenceMatcher::new("/03-05")?;
        assert!(!matcher.is_match(Some(&Occurrence::new("00")?)));
        assert!(!matcher.is_match(Some(&Occurrence::new("02")?)));
        assert!(matcher.is_match(Some(&Occurrence::new("03")?)));
        assert!(matcher.is_match(Some(&Occurrence::new("04")?)));
        assert!(matcher.is_match(Some(&Occurrence::new("05")?)));
        assert!(!matcher.is_match(Some(&Occurrence::new("06")?)));

        // Any
        let matcher = OccurrenceMatcher::new("/*")?;
        assert!(matcher.is_match(Some(&Occurrence::new("00")?)));
        assert!(matcher.is_match(Some(&Occurrence::new("01")?)));
        assert!(matcher.is_match(None));

        // None
        let matcher = OccurrenceMatcher::None;
        assert!(matcher.is_match(Some(&Occurrence::new("00")?)));
        assert!(!matcher.is_match(Some(&Occurrence::new("01")?)));
        assert!(matcher.is_match(None));

        // Error
        assert!(OccurrenceMatcher::new("/0A").is_err());
        assert!(OccurrenceMatcher::new("/05-03").is_err());
        assert!(OccurrenceMatcher::new("/05-05").is_err());
        assert!(OccurrenceMatcher::new("/05-0A").is_err());
        assert!(OccurrenceMatcher::new("/A").is_err());

        Ok(())
    }

    #[test]
    fn test_occurrence_matcher_to_string() -> TestResult {
        assert_eq!(OccurrenceMatcher::new("/01")?.to_string(), "/01");
        assert_eq!(
            OccurrenceMatcher::new("/01-04")?.to_string(),
            "/01-04"
        );
        assert_eq!(OccurrenceMatcher::new("/*")?.to_string(), "/*");
        assert_eq!(OccurrenceMatcher::new("")?.to_string(), "");

        Ok(())
    }

    #[quickcheck]
    fn occurrence_matcher_quickcheck1(occurrence: Occurrence) -> bool {
        OccurrenceMatcher::from(occurrence.clone())
            .is_match(Some(&occurrence))
    }

    #[quickcheck]
    fn occurrence_matcher_quickcheck2(occurrence: Occurrence) -> bool {
        OccurrenceMatcher::Any.is_match(Some(&occurrence))
    }
}
