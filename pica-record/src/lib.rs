//! This crate provides the low-level primitives to work with
//! bibliographic records encoded in PICA+. There exists a read-only
//! (immutable) and mutable variant of each primitive.

mod error;
mod field;
pub mod io;
mod occurrence;
mod record;
mod subfield;
mod tag;

pub use error::ParsePicaError;
pub use field::{Field, FieldMut, FieldRef};
pub use occurrence::{Occurrence, OccurrenceMut, OccurrenceRef};
pub use record::{
    ByteRecord, Record, RecordMut, RecordRef, StringRecord,
};
pub use subfield::{Subfield, SubfieldMut, SubfieldRef};
pub use tag::{Tag, TagMut, TagRef};

/// Parsers recognizing low-level primitives (e.g. subfield codes).
#[rustfmt::skip]
pub mod parser {
    pub(crate) const LF: u8 = b'\x0A'; // Line Feed
    pub(crate) const RS: u8 = b'\x1E'; // Record Separator
    pub(crate) const US: u8 = b'\x1F'; // Unit Separator
    pub(crate) const SP: u8 = b'\x20'; // Space

    /// Holds the result of a parsing function.
    ///
    /// It takes a byte slice as input and uses `nom::Err<()>` as error
    /// variant. The type only depends the output type `O`.
    pub type ParseResult<'a, O> = Result<(&'a [u8], O), nom::Err<()>>;

    pub use super::field::parse_field;
    pub use super::occurrence::parse_occurrence;
    pub use super::occurrence::parse_occurrence_digits;
    pub use super::subfield::parse_subfield_code;
    pub use super::subfield::parse_subfield_value;
    pub use super::tag::parse_tag;
}
