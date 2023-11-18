use std::ops::{BitAnd, BitOr, Not};

use bstr::ByteSlice;
use pica_record::Record;
#[cfg(feature = "serde")]
use serde::Deserialize;
use winnow::Parser;

use crate::common::BooleanOp;
use crate::field_matcher::parse_field_matcher;
use crate::{FieldMatcher, MatcherOptions, ParseMatcherError};

/// A Matcher that works on PICA+ [Records](pica_record::Record).
#[derive(Debug)]
pub struct RecordMatcher<'a> {
    pub(crate) field_matcher: FieldMatcher<'a>,
}

impl<'a> RecordMatcher<'a> {
    /// Create a new field matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::RecordMatcher;
    /// use pica_record::Record;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = RecordMatcher::new("003@?");
    ///     let record =
    ///         Record::new(vec![("003@", None, vec![('0', "abc")])]);
    ///
    ///     assert!(matcher.is_match(&record, &Default::default()));
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &'a T) -> Self {
        Self::try_from(data.as_ref()).expect("record matcher")
    }

    /// Returns `true` if the given record matches against the record
    /// matcher.
    pub fn is_match(
        &self,
        record: &Record,
        options: &MatcherOptions,
    ) -> bool {
        self.field_matcher.is_match(record.iter(), options)
    }
}

impl<'a> TryFrom<&'a [u8]> for RecordMatcher<'a> {
    type Error = ParseMatcherError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let matcher_str = value.to_str_lossy().to_string();

        parse_field_matcher
            .parse(value)
            .map(|field_matcher| RecordMatcher { field_matcher })
            .map_err(|_| {
                ParseMatcherError::InvalidRecordMatcher(matcher_str)
            })
    }
}

impl<'a> BitAnd for RecordMatcher<'a> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        RecordMatcher {
            field_matcher: FieldMatcher::Composite {
                lhs: Box::new(self.field_matcher),
                op: BooleanOp::And,
                rhs: Box::new(rhs.field_matcher),
            },
        }
    }
}

impl<'a> BitOr for RecordMatcher<'a> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        RecordMatcher {
            field_matcher: FieldMatcher::Composite {
                lhs: Box::new(self.field_matcher),
                op: BooleanOp::Or,
                rhs: Box::new(rhs.field_matcher),
            },
        }
    }
}

impl<'a> Not for RecordMatcher<'a> {
    type Output = Self;

    fn not(self) -> Self::Output {
        RecordMatcher {
            field_matcher: FieldMatcher::Not(Box::new(
                self.field_matcher,
            )),
        }
    }
}

#[cfg(feature = "serde")]
impl<'a, 'de: 'a> Deserialize<'de> for RecordMatcher<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &'de str = Deserialize::deserialize(deserializer)?;
        RecordMatcher::try_from(s.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}
