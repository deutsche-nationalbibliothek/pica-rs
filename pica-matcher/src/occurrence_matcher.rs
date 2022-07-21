use std::fmt;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, satisfy};
use nom::combinator::{
    all_consuming, cut, map, recognize, success, value, verify,
};
use nom::multi::many_m_n;
use nom::sequence::{preceded, separated_pair};
use nom::Finish;

use pica_core::{Occurrence, ParseResult};

use crate::ParseError;

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
            Self::Some(o) => write!(f, "{}", o),
            Self::Range(from, to) => {
                let to = to.to_string();
                write!(f, "{}-{}", from, &to[1..to.len()])
            }
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
    /// use pica_matcher::OccurrenceMatcher;
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
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, ParseError> {
        let data = data.as_ref();

        match all_consuming(parse_occurrence_matcher)(data.as_bytes()).finish()
        {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(ParseError::InvalidOccurrenceMatcher),
        }
    }

    /// Returns true, if and only if the given value matches against
    /// the occurrence matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::Occurrence;
    /// use pica_matcher::OccurrenceMatcher;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = OccurrenceMatcher::new("/01-09")?;
    ///     assert!(matcher.is_match(Some(&Occurrence::from_str("/03")?)));
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
                matches!(self, OccurrenceMatcher::Any | OccurrenceMatcher::None)
            }
        }
    }
}

impl From<Occurrence> for OccurrenceMatcher {
    fn from(occurrence: Occurrence) -> Self {
        Self::Some(occurrence)
    }
}

#[inline]
fn parse_occurrence_digits(i: &[u8]) -> ParseResult<&[u8]> {
    recognize(many_m_n(2, 3, satisfy(|c| c.is_ascii_digit())))(i)
}

pub fn parse_occurrence_matcher(i: &[u8]) -> ParseResult<OccurrenceMatcher> {
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
                        |(min, max)| min.len() == max.len() && min < max,
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
    use crate::TestResult;
    use std::str::FromStr;

    #[test]
    fn test_occurrence_matcher() -> TestResult {
        // Some
        let matcher = OccurrenceMatcher::new("/03")?;
        assert!(!matcher.is_match(Some(&Occurrence::from_str("/00")?)));
        assert!(!matcher.is_match(Some(&Occurrence::from_str("/02")?)));
        assert!(matcher.is_match(Some(&Occurrence::from_str("/03")?)));
        assert!(!matcher.is_match(Some(&Occurrence::from_str("/04")?)));

        // Range
        let matcher = OccurrenceMatcher::new("/03-05")?;
        assert!(!matcher.is_match(Some(&Occurrence::from_str("/00")?)));
        assert!(!matcher.is_match(Some(&Occurrence::from_str("/02")?)));
        assert!(matcher.is_match(Some(&Occurrence::from_str("/03")?)));
        assert!(matcher.is_match(Some(&Occurrence::from_str("/04")?)));
        assert!(matcher.is_match(Some(&Occurrence::from_str("/05")?)));
        assert!(!matcher.is_match(Some(&Occurrence::from_str("/06")?)));

        // Any
        let matcher = OccurrenceMatcher::new("/*")?;
        assert!(matcher.is_match(Some(&Occurrence::from_str("/00")?)));
        assert!(matcher.is_match(Some(&Occurrence::from_str("/01")?)));
        assert!(matcher.is_match(None));

        // None
        let matcher = OccurrenceMatcher::None;
        assert!(matcher.is_match(Some(&Occurrence::from_str("/00")?)));
        assert!(!matcher.is_match(Some(&Occurrence::from_str("/01")?)));
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
        assert_eq!(OccurrenceMatcher::new("/01-04")?.to_string(), "/01-04");
        assert_eq!(OccurrenceMatcher::new("/*")?.to_string(), "/*");
        assert_eq!(OccurrenceMatcher::new("")?.to_string(), "");

        Ok(())
    }
}
