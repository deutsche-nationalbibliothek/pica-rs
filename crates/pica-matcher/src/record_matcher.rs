use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not,
};
use std::str::FromStr;

use bstr::ByteSlice;
use pica_record::RecordRef;
#[cfg(feature = "serde")]
use serde::Deserialize;
use winnow::Parser;

use crate::common::BooleanOp;
use crate::field_matcher::parse_field_matcher;
use crate::{FieldMatcher, MatcherOptions, ParseMatcherError};

/// A Matcher that works on PICA+ [Records](pica_record::Record).
#[derive(Debug)]
pub struct RecordMatcher {
    pub(crate) field_matcher: FieldMatcher,
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
    ///     let matcher = RecordMatcher::new("003@?");
    ///     let record =
    ///         RecordRef::new(vec![("003@", None, vec![('0', "abc")])]);
    ///
    ///     assert!(matcher.is_match(&record, &Default::default()));
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &T) -> Self {
        Self::try_from(data.as_ref()).expect("record matcher")
    }

    /// Returns `true` if the given record matches against the record
    /// matcher.
    pub fn is_match(
        &self,
        record: &RecordRef,
        options: &MatcherOptions,
    ) -> bool {
        self.field_matcher.is_match(record.iter(), options)
    }
}

impl TryFrom<&[u8]> for RecordMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let matcher_str = value.to_str_lossy().to_string();

        parse_field_matcher
            .parse(value)
            .map(|field_matcher| RecordMatcher { field_matcher })
            .map_err(|_| {
                ParseMatcherError::InvalidRecordMatcher(matcher_str)
            })
    }
}

impl FromStr for RecordMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

impl TryFrom<&String> for RecordMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_bytes())
    }
}

impl BitAnd for RecordMatcher {
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

impl BitAndAssign for RecordMatcher {
    fn bitand_assign(&mut self, rhs: Self) {
        self.field_matcher = FieldMatcher::Composite {
            lhs: Box::new(self.field_matcher.clone()),
            op: BooleanOp::And,
            rhs: Box::new(rhs.field_matcher),
        };
    }
}

impl BitOr for RecordMatcher {
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

impl BitOrAssign for RecordMatcher {
    fn bitor_assign(&mut self, rhs: Self) {
        self.field_matcher = FieldMatcher::Composite {
            lhs: Box::new(self.field_matcher.clone()),
            op: BooleanOp::Or,
            rhs: Box::new(rhs.field_matcher),
        };
    }
}

impl BitXor for RecordMatcher {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        RecordMatcher {
            field_matcher: FieldMatcher::Composite {
                lhs: Box::new(self.field_matcher),
                op: BooleanOp::Xor,
                rhs: Box::new(rhs.field_matcher),
            },
        }
    }
}

impl BitXorAssign for RecordMatcher {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.field_matcher = FieldMatcher::Composite {
            lhs: Box::new(self.field_matcher.clone()),
            op: BooleanOp::Xor,
            rhs: Box::new(rhs.field_matcher),
        };
    }
}

impl Not for RecordMatcher {
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
impl<'de> Deserialize<'de> for RecordMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        RecordMatcher::try_from(s.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}
