use std::str::FromStr;

use bstr::{BStr, ByteSlice};
use pica_record::parser::parse_occurrence_digits;
use pica_record::{Occurrence, OccurrenceRef};
use winnow::combinator::{alt, preceded, separated_pair, success};
use winnow::{PResult, Parser};

use crate::ParseMatcherError;

/// A matcher that matches against PICA+
/// [Occurrence](`pica_record::Occurrence`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OccurrenceMatcher {
    Any,
    Exact(Occurrence),
    Range(Occurrence, Occurrence),
    None,
}

impl OccurrenceMatcher {
    /// Create a new occurrence matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::OccurrenceMatcher;
    /// use pica_record::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = OccurrenceMatcher::new("/01-02");
    ///     assert!(matches!(matcher, OccurrenceMatcher::Range(_, _)));
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn new<T: ?Sized + AsRef<[u8]>>(value: &T) -> Self {
        Self::try_from(value.as_ref()).expect("occurrence matcher")
    }

    /// Returns `true` if the given occurrence matches against the
    /// matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::OccurrenceMatcher;
    /// use pica_record::OccurrenceRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = OccurrenceMatcher::new("/01-03");
    ///     assert!(matcher.is_match(&OccurrenceRef::new("02")));
    ///     assert!(!matcher.is_match(&OccurrenceRef::new("04")));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, other: &OccurrenceRef) -> bool {
        match self {
            Self::Any => true,
            Self::None => other == b"00",
            Self::Exact(rhs) => other == rhs,
            Self::Range(min, max) => {
                (other.as_bytes() >= min.as_bytes())
                    && (other.as_bytes() <= max.as_bytes())
            }
        }
    }

    #[cfg(test)]
    fn exact<T: ?Sized + AsRef<[u8]>>(value: &T) -> Self {
        Self::Exact(OccurrenceRef::new(value).into())
    }

    #[cfg(test)]
    fn range<T: ?Sized + AsRef<[u8]>>(min: &T, max: &T) -> Self {
        Self::Range(
            OccurrenceRef::new(min).into(),
            OccurrenceRef::new(max).into(),
        )
    }
}

#[inline]
fn parse_occurrence_range(i: &mut &[u8]) -> PResult<OccurrenceMatcher> {
    separated_pair(
        parse_occurrence_digits,
        '-',
        parse_occurrence_digits,
    )
    .verify(|(min, max)| min.len() == max.len() && min < max)
    .map(|(min, max)| {
        OccurrenceMatcher::Range(
            OccurrenceRef::from_unchecked(min).into(),
            OccurrenceRef::from_unchecked(max).into(),
        )
    })
    .parse_next(i)
}

#[inline]
fn parse_occurrence_exact(i: &mut &[u8]) -> PResult<OccurrenceMatcher> {
    parse_occurrence_digits
        .verify(|x: &BStr| x != "00")
        .map(|value| {
            OccurrenceMatcher::Exact(
                OccurrenceRef::from_unchecked(value).into(),
            )
        })
        .parse_next(i)
}

pub fn parse_occurrence_matcher(
    i: &mut &[u8],
) -> PResult<OccurrenceMatcher> {
    alt((
        preceded(
            '/',
            alt((
                parse_occurrence_range,
                parse_occurrence_exact,
                "00".value(OccurrenceMatcher::None),
                '*'.value(OccurrenceMatcher::Any),
            )),
        ),
        success(OccurrenceMatcher::None),
    ))
    .parse_next(i)
}

impl TryFrom<&[u8]> for OccurrenceMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_occurrence_matcher.parse(value).map_err(|_| {
            ParseMatcherError::InvalidOccurrenceMatcher(
                value.to_str_lossy().to_string(),
            )
        })
    }
}

impl FromStr for OccurrenceMatcher {
    type Err = ParseMatcherError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

impl From<OccurrenceRef<'_>> for OccurrenceMatcher {
    fn from(value: OccurrenceRef) -> Self {
        OccurrenceMatcher::Exact(value.into())
    }
}

impl PartialEq<OccurrenceRef<'_>> for OccurrenceMatcher {
    fn eq(&self, other: &OccurrenceRef) -> bool {
        self.is_match(other)
    }
}

impl PartialEq<OccurrenceMatcher> for OccurrenceRef<'_> {
    fn eq(&self, matcher: &OccurrenceMatcher) -> bool {
        matcher.is_match(self)
    }
}

impl PartialEq<Option<&OccurrenceRef<'_>>> for OccurrenceMatcher {
    fn eq(&self, other: &Option<&OccurrenceRef>) -> bool {
        match other {
            Some(occurrence) => self.is_match(occurrence),
            None => matches!(self, Self::Any | Self::None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_occurrence_matcher() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    super::parse_occurrence_matcher
                        .parse($input)
                        .unwrap(),
                    $expected
                );
            };
        }

        parse_success!(b"/*", OccurrenceMatcher::Any);
        parse_success!(b"/00", OccurrenceMatcher::None);
        parse_success!(b"/01", OccurrenceMatcher::exact("01"));
        parse_success!(b"/01-03", OccurrenceMatcher::range("01", "03"));
        parse_success!(b"", OccurrenceMatcher::None);

        macro_rules! parse_error {
            ($input:expr) => {
                assert!(super::parse_occurrence_matcher
                    .parse($input)
                    .is_err());
            };
        }

        parse_error!(b"/03-01");
        parse_error!(b"/0001");
        parse_error!(b"/0A");
        parse_error!(b"/A");
    }
}
