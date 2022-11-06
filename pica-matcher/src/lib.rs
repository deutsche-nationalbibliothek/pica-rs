//! This crate provides various matcher to filter PICA+ records, fields
//! or subfields.

mod common;
mod error;
mod flags;
mod occurrence_matcher;
mod subfield_matcher;
mod tag_matcher;

pub use error::ParseMatcherError;
pub use flags::MatcherFlags;
pub use occurrence_matcher::OccurrenceMatcher;
pub use subfield_matcher::SubfieldMatcher;
pub use tag_matcher::TagMatcher;
