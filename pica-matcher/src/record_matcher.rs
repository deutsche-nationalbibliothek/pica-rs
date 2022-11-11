use std::fmt::{self, Display};
use std::str::FromStr;

use nom::combinator::all_consuming;
use nom::Finish;
use pica_record::Record;
#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::field_matcher::parse_field_matcher;
use crate::{FieldMatcher, MatcherOptions, ParseMatcherError};

/// A Matcher that works on PICA+ [Records](pica_record::Record).
#[derive(Debug, PartialEq, Eq)]
pub struct RecordMatcher {
    field_matcher: FieldMatcher,
    matcher_str: String,
}

impl RecordMatcher {
    /// Create a new field matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::RecordMatcher;
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = RecordMatcher::new("003@?")?;
    ///     let record =
    ///         RecordRef::new(vec![("003@", None, vec![('0', "abc")])]);
    ///
    ///     assert!(matcher.is_match(&record, &Default::default()));
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_field_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidRecordMatcher(data.into())
            })
            .map(|(_, matcher)| Self {
                field_matcher: matcher,
                matcher_str: data.into(),
            })
    }

    /// Returns `true` if the given record matches against the record
    /// matcher.
    pub fn is_match<T: AsRef<[u8]>>(
        &self,
        record: &Record<T>,
        options: &MatcherOptions,
    ) -> bool {
        self.field_matcher.is_match(record.iter(), options)
    }
}

impl Display for RecordMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.matcher_str)
    }
}

impl FromStr for RecordMatcher {
    type Err = ParseMatcherError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for RecordMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        RecordMatcher::from_str(&s).map_err(serde::de::Error::custom)
    }
}
