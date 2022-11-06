//! This crate provides various matcher to filter PICA+ records, fields
//! or subfields.

mod error;
mod occurrence_matcher;
mod subfield_matcher;
mod tag_matcher;

pub use error::ParseMatcherError;
pub use occurrence_matcher::OccurrenceMatcher;
pub use subfield_matcher::SubfieldMatcher;
pub use tag_matcher::TagMatcher;
