mod common;
mod error;
mod field_matcher;
mod flags;
mod occurrence_matcher;
mod ops;
mod record_matcher;
mod subfield_list_matcher;
mod subfield_matcher;
mod tag_matcher;

pub use error::ParseError;
pub use field_matcher::FieldMatcher;
pub use flags::MatcherFlags;
pub use occurrence_matcher::OccurrenceMatcher;
pub use ops::{BooleanOp, ComparisonOp};
pub use record_matcher::RecordMatcher;
pub use subfield_list_matcher::SubfieldListMatcher;
pub use subfield_matcher::SubfieldMatcher;
pub use tag_matcher::TagMatcher;

/// Parser combinator for parsing matchers.
pub mod parser {
    pub use crate::field_matcher::parse_field_matcher;
    pub use crate::occurrence_matcher::parse_occurrence_matcher;
    pub use crate::record_matcher::parse_record_matcher;
    pub use crate::subfield_list_matcher::parse_subfield_list_matcher;
    pub use crate::subfield_matcher::parse_subfield_matcher;
    pub use crate::tag_matcher::parse_tag_matcher;
}

/// Holds the result of a test function.
#[cfg(test)]
pub(crate) type TestResult = Result<(), Box<dyn std::error::Error>>;
