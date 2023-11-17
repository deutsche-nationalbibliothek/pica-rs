//! This crate provides various matcher to filter PICA+ records, fields
//! or subfields.

mod common;
// mod subfield_matcher;
mod error;
// pub mod field_matcher;
mod occurrence_matcher;
// mod options;
// mod record_matcher;
mod tag_matcher;

pub use error::ParseMatcherError;
// pub use field_matcher::FieldMatcher;
pub use occurrence_matcher::OccurrenceMatcher;
// pub use options::MatcherOptions;
// pub use record_matcher::RecordMatcher;
// pub use subfield_matcher::SubfieldMatcher;
pub use tag_matcher::TagMatcher;

// /// Parsers recognizing matcher for PICA+ primitives.
// pub mod parser {
//     pub use super::field_matcher::parse_field_matcher;
//     pub use super::occurrence_matcher::parse_occurrence_matcher;
//     pub use super::subfield_matcher::parse_subfield_matcher;
//     pub use super::tag_matcher::parse_tag_matcher;
// }
