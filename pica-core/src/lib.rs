mod error;
mod occurrence;
mod tag;
mod types;

pub use error::ParseError;
pub use occurrence::{Occurrence, OccurrenceRef};
pub use tag::{Tag, TagRef};
pub use types::ParseResult;

/// Parser combinator for parsing PICA+ records.
pub mod parser {
    pub use super::occurrence::parse_occurrence;
    pub use super::tag::parse_tag;
}
