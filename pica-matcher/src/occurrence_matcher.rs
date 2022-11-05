use std::fmt::{self, Display};

use bstr::BStr;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, map, value, verify};
use nom::sequence::{preceded, separated_pair};
use nom::Finish;
use pica_record::parser::{parse_occurrence_digits, ParseResult};
use pica_record::{Occurrence, OccurrenceMut};

use crate::ParseMatcherError;

/// A matcher that matches against PICA+
/// [Occurrence](`pica_record::Occurrence`).
#[derive(Debug)]
pub struct OccurrenceMatcher {
    kind: OccurrenceMatcherKind,
    matcher_str: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum OccurrenceMatcherKind {
    Any,
    Some(OccurrenceMut),
    Range(OccurrenceMut, OccurrenceMut),
    None,
}

impl OccurrenceMatcher {
    /// Create a new tag matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::TagMatcher;
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = TagMatcher::new("003@")?;
    ///     assert_eq!(matcher, TagRef::new("003@"));
    ///
    ///     # assert!(TagMatcher::new("003!").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T>(expr: T) -> Result<Self, ParseMatcherError>
    where
        T: AsRef<[u8]> + Display,
    {
        all_consuming(parse_occurrence_matcher_kind)(expr.as_ref())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidOccurrenceMatcher(
                    expr.to_string(),
                )
            })
            .map(|(_, kind)| Self {
                matcher_str: expr.to_string(),
                kind,
            })
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
    ///     let matcher = OccurrenceMatcher::new("/01-03")?;
    ///     assert!(matcher.is_match(&OccurrenceRef::new("02")));
    ///     assert!(!matcher.is_match(&OccurrenceRef::new("04")));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match<T: AsRef<[u8]>>(
        &self,
        occurrence: &Occurrence<T>,
    ) -> bool {
        match &self.kind {
            OccurrenceMatcherKind::Any => true,
            OccurrenceMatcherKind::None => occurrence == "00",
            OccurrenceMatcherKind::Some(rhs) => occurrence == rhs,
            OccurrenceMatcherKind::Range(min, max) => {
                (occurrence >= min) && (occurrence <= max)
            }
        }
    }
}

impl Display for OccurrenceMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.matcher_str)
    }
}

impl PartialEq for OccurrenceMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<T: AsRef<[u8]>> PartialEq<Occurrence<T>> for OccurrenceMatcher {
    fn eq(&self, other: &Occurrence<T>) -> bool {
        self.is_match(other)
    }
}

impl<T: AsRef<[u8]>> PartialEq<Option<Occurrence<T>>>
    for OccurrenceMatcher
{
    fn eq(&self, other: &Option<Occurrence<T>>) -> bool {
        match other {
            Some(occurrence) => self.is_match(occurrence),
            None => matches!(
                self.kind,
                OccurrenceMatcherKind::Any
                    | OccurrenceMatcherKind::None
            ),
        }
    }
}

impl<T: AsRef<[u8]>> PartialEq<OccurrenceMatcher> for Occurrence<T> {
    fn eq(&self, matcher: &OccurrenceMatcher) -> bool {
        matcher.is_match(self)
    }
}

impl From<OccurrenceMut> for OccurrenceMatcherKind {
    fn from(value: OccurrenceMut) -> Self {
        OccurrenceMatcherKind::Some(value)
    }
}

fn parse_occurrence_matcher_kind(
    i: &[u8],
) -> ParseResult<OccurrenceMatcherKind> {
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
                    OccurrenceMatcherKind::Range(
                        OccurrenceMut::from_unchecked(min),
                        OccurrenceMut::from_unchecked(max),
                    )
                },
            ),
            map(
                verify(parse_occurrence_digits, |x: &BStr| {
                    x.to_vec() != b"00"
                }),
                |value| OccurrenceMut::from_unchecked(value).into(),
            ),
            value(OccurrenceMatcherKind::None, tag("00")),
            value(OccurrenceMatcherKind::Any, char('*')),
        ))),
    )(i)
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;
    use pica_record::OccurrenceRef;

    use super::*;

    #[test]
    fn test_parse_occurrence_matcher_kind() -> anyhow::Result<()> {
        assert_done_and_eq!(
            parse_occurrence_matcher_kind(b"/*"),
            OccurrenceMatcherKind::Any
        );

        assert_done_and_eq!(
            parse_occurrence_matcher_kind(b"/00"),
            OccurrenceMatcherKind::None
        );

        assert_done_and_eq!(
            parse_occurrence_matcher_kind(b"/01"),
            OccurrenceMatcherKind::Some(OccurrenceMut::from_bytes(
                b"/01"
            )?)
        );

        assert_done_and_eq!(
            parse_occurrence_matcher_kind(b"/01-03"),
            OccurrenceMatcherKind::Range(
                OccurrenceMut::from_bytes(b"/01")?,
                OccurrenceMut::from_bytes(b"/03")?,
            )
        );

        assert_error!(parse_occurrence_matcher_kind(b"/0A"));
        assert_error!(parse_occurrence_matcher_kind(b"/A"));

        Ok(())
    }

    #[test]
    fn test_is_match() -> anyhow::Result<()> {
        let matcher = OccurrenceMatcher::new("/01")?;
        assert!(!matcher.is_match(&OccurrenceRef::from_bytes(b"/00")?));
        assert!(matcher.is_match(&OccurrenceRef::from_bytes(b"/01")?));

        let matcher = OccurrenceMatcher::new("/01-03")?;
        assert!(!matcher.is_match(&OccurrenceRef::from_bytes(b"/00")?));
        assert!(matcher.is_match(&OccurrenceRef::from_bytes(b"/01")?));
        assert!(matcher.is_match(&OccurrenceRef::from_bytes(b"/02")?));
        assert!(matcher.is_match(&OccurrenceRef::from_bytes(b"/03")?));
        assert!(!matcher.is_match(&OccurrenceRef::from_bytes(b"/04")?));

        let matcher = OccurrenceMatcher::new("/*")?;
        assert!(matcher.is_match(&OccurrenceRef::from_bytes(b"/00")?));
        assert!(matcher.is_match(&OccurrenceRef::from_bytes(b"/01")?));

        let matcher = OccurrenceMatcher::new("/00")?;
        assert!(matcher.is_match(&OccurrenceRef::from_bytes(b"/00")?));
        assert!(!matcher.is_match(&OccurrenceRef::from_bytes(b"/01")?));

        Ok(())
    }

    #[test]
    fn test_partial_eq() -> anyhow::Result<()> {
        let matcher = OccurrenceMatcher::new("/01")?;
        assert_ne!(matcher, OccurrenceRef::from_bytes(b"/00")?);
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/01")?);
        assert_ne!(matcher, Option::<OccurrenceRef>::None);

        let matcher = OccurrenceMatcher::new("/01-03")?;
        assert_ne!(matcher, OccurrenceRef::from_bytes(b"/00")?);
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/01")?);
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/02")?);
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/03")?);
        assert_ne!(matcher, OccurrenceRef::from_bytes(b"/04")?);
        assert_ne!(matcher, Option::<OccurrenceRef>::None);

        let matcher = OccurrenceMatcher::new("/*")?;
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/000")?);
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/00")?);
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/001")?);
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/01")?);
        assert_eq!(matcher, Option::<OccurrenceRef>::None);

        let matcher = OccurrenceMatcher::new("/00")?;
        assert_eq!(matcher, OccurrenceRef::from_bytes(b"/00")?);
        assert_ne!(matcher, OccurrenceRef::from_bytes(b"/01")?);
        assert_eq!(matcher, Option::<OccurrenceRef>::None);

        Ok(())
    }
}
