use std::fmt::{self, Display};

use winnow::Parser;

use super::parse::parse_occurrence_matcher;
use super::ParseMatcherError;
use crate::primitives::{Occurrence, OccurrenceRef};

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
    /// assert!(matcher.is_match(&OccurrenceRef::new("02")?));
    /// assert!(!matcher.is_match(&OccurrenceRef::new("04")?));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
