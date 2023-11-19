//! This crate provides the low-level primitives to work with
//! bibliographic records encoded in PICA+.

mod error;
mod field;
pub mod io;
mod level;
mod occurrence;
mod record;
mod subfield;
mod tag;

pub use error::ParsePicaError;
pub use field::{Field, FieldRef};
pub use level::Level;
pub use occurrence::{Occurrence, OccurrenceRef};
pub use record::{ByteRecord, Record, RecordRef, StringRecord};
pub use subfield::SubfieldRef;
pub use tag::{Tag, TagRef};

/// Parsers recognizing low-level primitives (e.g. subfield codes).
#[rustfmt::skip]
pub mod parser {
    pub use super::occurrence::parse_occurrence_digits;
    pub use super::subfield::parse_subfield_code;
    pub use super::tag::parse_tag;
}
