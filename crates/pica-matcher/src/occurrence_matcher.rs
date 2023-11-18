use bstr::{BStr, ByteSlice};
use pica_record::parser::parse_occurrence_digits;
use pica_record::Occurrence;
use winnow::combinator::{alt, preceded, separated_pair, success};
use winnow::{PResult, Parser};

use crate::ParseMatcherError;

/// A matcher that matches against PICA+
/// [Occurrence](`pica_record::Occurrence`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OccurrenceMatcher<'a> {
    Any,
    Exact(Occurrence<'a>),
    Range(Occurrence<'a>, Occurrence<'a>),
    None,
}

impl<'a> OccurrenceMatcher<'a> {
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
    pub fn new<T: ?Sized + AsRef<[u8]>>(value: &'a T) -> Self {
        Self::try_from(value.as_ref()).expect("occurrence matcher")
    }

    /// Returns `true` if the given occurrence matches against the
    /// matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::OccurrenceMatcher;
    /// use pica_record::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = OccurrenceMatcher::new("/01-03");
    ///     assert!(matcher.is_match(&Occurrence::new("02")));
    ///     assert!(!matcher.is_match(&Occurrence::new("04")));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, other: &Occurrence) -> bool {
        match self {
            Self::Any => true,
            Self::None => other == b"00",
            Self::Exact(rhs) => other == rhs,
            Self::Range(min, max) => (other >= min) && (other <= max),
        }
    }

    #[cfg(test)]
    fn exact<T: ?Sized + AsRef<[u8]>>(value: &'a T) -> Self {
        Self::Exact(Occurrence::new(value))
    }

    #[cfg(test)]
    fn range<T: ?Sized + AsRef<[u8]>>(min: &'a T, max: &'a T) -> Self {
        Self::Range(Occurrence::new(min), Occurrence::new(max))
    }
}

#[inline]
fn parse_occurrence_range<'a>(
    i: &mut &'a [u8],
) -> PResult<OccurrenceMatcher<'a>> {
    separated_pair(
        parse_occurrence_digits,
        '-',
        parse_occurrence_digits,
    )
    .verify(|(min, max)| min.len() == max.len() && min < max)
    .map(|(min, max)| {
        OccurrenceMatcher::Range(
            Occurrence::from_unchecked(min),
            Occurrence::from_unchecked(max),
        )
    })
    .parse_next(i)
}

#[inline]
fn parse_occurrence_exact<'a>(
    i: &mut &'a [u8],
) -> PResult<OccurrenceMatcher<'a>> {
    parse_occurrence_digits
        .verify(|x: &BStr| x != "00")
        .map(|value| {
            OccurrenceMatcher::Exact(Occurrence::from_unchecked(value))
        })
        .parse_next(i)
}

pub fn parse_occurrence_matcher<'a>(
    i: &mut &'a [u8],
) -> PResult<OccurrenceMatcher<'a>> {
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

impl<'a> TryFrom<&'a [u8]> for OccurrenceMatcher<'a> {
    type Error = ParseMatcherError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        parse_occurrence_matcher.parse(value).map_err(|_| {
            ParseMatcherError::InvalidOccurrenceMatcher(
                value.to_str_lossy().to_string(),
            )
        })
    }
}

impl<'a> TryFrom<&'a str> for OccurrenceMatcher<'a> {
    type Error = ParseMatcherError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(value.as_bytes())
    }
}

impl<'a> From<Occurrence<'a>> for OccurrenceMatcher<'a> {
    fn from(value: Occurrence<'a>) -> Self {
        OccurrenceMatcher::Exact(value)
    }
}

impl PartialEq<Occurrence<'_>> for OccurrenceMatcher<'_> {
    fn eq(&self, other: &Occurrence) -> bool {
        self.is_match(other)
    }
}

impl PartialEq<OccurrenceMatcher<'_>> for Occurrence<'_> {
    fn eq(&self, matcher: &OccurrenceMatcher) -> bool {
        matcher.is_match(self)
    }
}

impl PartialEq<Option<&Occurrence<'_>>> for OccurrenceMatcher<'_> {
    fn eq(&self, other: &Option<&Occurrence>) -> bool {
        match other {
            Some(occurrence) => self.is_match(occurrence),
            None => matches!(self, Self::Any | Self::None),
        }
    }
}

#[cfg(test)]
mod tests {
    use pica_record::Occurrence;

    use super::*;

    #[test]
    fn parse_occurrence_matcher() {
        use super::parse_occurrence_matcher;

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_occurrence_matcher.parse($input).unwrap(),
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
                assert!(parse_occurrence_matcher
                    .parse($input)
                    .is_err());
            };
        }

        parse_error!(b"/03-01");
        parse_error!(b"/0001");
        parse_error!(b"/0A");
        parse_error!(b"/A");
    }

    #[test]
    fn is_match() {
        let matcher = OccurrenceMatcher::new("/01");
        assert!(!matcher.is_match(&Occurrence::new("00")));
        assert!(matcher.is_match(&Occurrence::new("01")));

        let matcher = OccurrenceMatcher::new("/01-03");
        assert!(!matcher.is_match(&Occurrence::new("00")));
        assert!(matcher.is_match(&Occurrence::new("01")));
        assert!(matcher.is_match(&Occurrence::new("02")));
        assert!(matcher.is_match(&Occurrence::new("03")));
        assert!(!matcher.is_match(&Occurrence::new("04")));

        let matcher = OccurrenceMatcher::new("/*");
        assert!(matcher.is_match(&Occurrence::new("00")));
        assert!(matcher.is_match(&Occurrence::new("01")));

        let matcher = OccurrenceMatcher::new("/00");
        assert!(matcher.is_match(&Occurrence::new("00")));
        assert!(!matcher.is_match(&Occurrence::new("01")));
    }

    #[test]
    fn test_partial_eq() {
        let matcher = OccurrenceMatcher::new("/01");
        assert_ne!(matcher, Occurrence::new("00"));
        assert_eq!(matcher, Occurrence::new("01"));
        assert_ne!(matcher, Option::<Occurrence>::None.as_ref());

        let matcher = OccurrenceMatcher::new("/01-03");
        assert_ne!(matcher, Occurrence::new("00"));
        assert_eq!(matcher, Occurrence::new("01"));
        assert_eq!(matcher, Occurrence::new("02"));
        assert_eq!(matcher, Occurrence::new("03"));
        assert_ne!(matcher, Occurrence::new("04"));
        assert_ne!(matcher, Option::<Occurrence>::None.as_ref());

        let matcher = OccurrenceMatcher::new("/*");
        assert_eq!(matcher, Occurrence::new("000"));
        assert_eq!(matcher, Occurrence::new("00"));
        assert_eq!(matcher, Occurrence::new("001"));
        assert_eq!(matcher, Occurrence::new("01"));
        assert_eq!(matcher, Option::<Occurrence>::None.as_ref());

        let matcher = OccurrenceMatcher::new("/00");
        assert_eq!(matcher, Occurrence::new("00"));
        assert_ne!(matcher, Occurrence::new("01"));
        assert_eq!(matcher, Option::<Occurrence>::None.as_ref());
    }
}
