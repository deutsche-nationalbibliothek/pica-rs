//! This crate provides various matcher to filter PICA+ records, fields
//! or subfields.

mod error;
mod tag_matcher;

pub use error::ParseMatcherError;
pub use tag_matcher::TagMatcher;
