//! Matcher that can be applied on a list of [OccurrenceRef].

use std::fmt::{self, Display};

use winnow::combinator::{alt, empty, preceded, separated_pair};
use winnow::prelude::*;

use super::ParseMatcherError;
use crate::primitives::parse::parse_occurrence_ref;
use crate::primitives::{Occurrence, OccurrenceRef};

/// A matcher that matches against a [OccurrenceRef].
#[derive(Debug, Clone, PartialEq)]
pub enum OccurrenceMatcher {
    Exact(Occurrence),
    Range(Occurrence, Occurrence),
    None,
    Any,
}

impl OccurrenceMatcher {
    /// Creates a new [OccurrenceMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// occurrence matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::OccurrenceMatcher;
    ///
    /// let _matcher = OccurrenceMatcher::new("/01")?;
    /// let _matcher = OccurrenceMatcher::new("/01-09")?;
    /// let _matcher = OccurrenceMatcher::new("/001")?;
    /// let _matcher = OccurrenceMatcher::new("/*")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_occurrence_matcher.parse(matcher.as_bytes()).map_err(
            |_| {
                ParseMatcherError(format!(
                    "invalid occurrence matcher '{matcher}'"
                ))
            },
        )
    }

    /// Returns `true` if the given occurrence matches against the
    /// matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::OccurrenceMatcher;
    /// use pica_record::primitives::OccurrenceRef;
    ///
    /// let matcher = OccurrenceMatcher::new("/01-03")?;
    /// assert!(matcher.is_match(Some(&OccurrenceRef::new("02")?)));
    /// assert!(!matcher.is_match(Some(&OccurrenceRef::new("04")?)));
    ///
    /// let matcher = OccurrenceMatcher::new("/*")?;
    /// assert!(matcher.is_match(None));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_match(&self, other: Option<&OccurrenceRef>) -> bool {
        match other {
            None => matches!(self, Self::Any | Self::None),
            Some(occ) => match self {
                Self::Any => true,
                Self::None => occ == b"00",
                Self::Exact(rhs) => occ == rhs,
                Self::Range(min, max) => {
                    (occ.as_bytes() >= min.as_bytes())
                        && (occ.as_bytes() <= max.as_bytes())
                }
            },
        }
    }
}

impl Display for OccurrenceMatcher {
    /// Formats a [OccurrenceMatcher] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::OccurrenceMatcher;
    ///
    /// let matcher = OccurrenceMatcher::new("/01-03")?;
    /// assert_eq!(matcher.to_string(), "/01-03");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(o) => write!(f, "/{o}")?,
            Self::Range(min, max) => write!(f, "/{min}-{max}")?,
            Self::Any => write!(f, "/*")?,
            Self::None => (),
        }

        Ok(())
    }
}

#[inline]
fn parse_occurrence_matcher_inner(
    i: &mut &[u8],
) -> ModalResult<Occurrence> {
    parse_occurrence_ref.map(Occurrence::from).parse_next(i)
}

#[inline]
fn parse_occurrence_matcher_exact(
    i: &mut &[u8],
) -> ModalResult<OccurrenceMatcher> {
    parse_occurrence_matcher_inner
        .verify(|occurrence| occurrence.as_bytes() != b"00")
        .map(OccurrenceMatcher::Exact)
        .parse_next(i)
}

#[inline]
fn parse_occurrence_matcher_range(
    i: &mut &[u8],
) -> ModalResult<OccurrenceMatcher> {
    separated_pair(
        parse_occurrence_matcher_inner,
        '-',
        parse_occurrence_matcher_inner,
    )
    .verify(|(min, max)| min.len() == max.len() && min < max)
    .map(|(min, max)| OccurrenceMatcher::Range(min, max))
    .parse_next(i)
}

pub(crate) fn parse_occurrence_matcher(
    i: &mut &[u8],
) -> ModalResult<OccurrenceMatcher> {
    alt((
        preceded(
            '/',
            alt((
                parse_occurrence_matcher_range,
                parse_occurrence_matcher_exact,
                "00".value(OccurrenceMatcher::None),
                "*".value(OccurrenceMatcher::Any),
            )),
        ),
        empty.value(OccurrenceMatcher::None),
    ))
    .parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_occurrence_matcher() -> anyhow::Result<()> {
        macro_rules! parse_success {
            ($i:expr, $o:expr) => {
                let o = parse_occurrence_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert_eq!(o.to_string(), $i);
                assert_eq!(o, $o);
            };
        }

        parse_success!("", OccurrenceMatcher::None);
        parse_success!("/*", OccurrenceMatcher::Any);

        parse_success!(
            "/01",
            OccurrenceMatcher::Exact(Occurrence::new("01")?)
        );

        parse_success!(
            "/999",
            OccurrenceMatcher::Exact(Occurrence::new("999")?)
        );

        parse_success!(
            "/01-99",
            OccurrenceMatcher::Range(
                Occurrence::new("01")?,
                Occurrence::new("99")?
            )
        );

        parse_success!(
            "/01-99",
            OccurrenceMatcher::Range(
                Occurrence::new("01")?,
                Occurrence::new("99")?
            )
        );

        assert_eq!(
            parse_occurrence_matcher.parse(b"/00").unwrap(),
            OccurrenceMatcher::None
        );

        Ok(())
    }
}
