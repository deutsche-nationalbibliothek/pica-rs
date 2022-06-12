mod error;
mod field;
mod occurrence;
mod subfield;
mod tag;
mod types;

pub use error::ParseError;
pub use field::{Field, FieldRef};
pub use occurrence::{Occurrence, OccurrenceRef};
pub use subfield::{Subfield, SubfieldRef};
pub use tag::{Tag, TagRef};
pub use types::ParseResult;

/// Parser combinator for parsing PICA+ records.
pub mod parser {
    pub(crate) const US: u8 = b'\x1F';
    pub(crate) const RS: u8 = b'\x1E';

    pub use super::field::parse_field;
    pub use super::occurrence::parse_occurrence;
    pub use super::subfield::{parse_subfield, parse_subfield_code};
    pub use super::tag::parse_tag;
}
