//! This crate provides various matcher to filter PICA+ records, fields
//! or subfields.

mod common;
mod error;
pub mod field_matcher;
mod occurrence_matcher;
mod options;
mod record_matcher;
pub mod subfield_matcher;
mod tag_matcher;

pub use error::ParseMatcherError;
pub use field_matcher::FieldMatcher;
pub use occurrence_matcher::OccurrenceMatcher;
pub use options::MatcherOptions;
pub use record_matcher::RecordMatcher;
pub use subfield_matcher::SubfieldMatcher;
pub use tag_matcher::TagMatcher;
