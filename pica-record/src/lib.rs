//! This crate provides the low-level primitives to work with
//! bibliographic records encoded in PICA+. There exists a read-only
//! (immutable) and mutable variant of each primitive.
#![feature(result_option_inspect)]

// #[cfg(test)]
// #[macro_use(quickcheck)]
// extern crate quickcheck_macros;

// mod arbitrary;
mod error;
// mod field;
// mod occurrence;
mod subfield;
mod tag;

pub use error::ParsePicaError;
// pub use field::{Field, FieldRef};
// pub use occurrence::{Occurrence, OccurrenceRef};
pub use subfield::{Subfield, SubfieldMut, SubfieldRef};
pub use tag::{Tag, TagMut, TagRef};

/// Parsers recognizing low-level primitives (e.g. subfield codes).
#[rustfmt::skip]
pub mod parser {
    pub(crate) const RS: u8 = b'\x1E'; // Record Separator
    pub(crate) const US: u8 = b'\x1F'; // Unit Separator
    // pub(crate) const SP: u8 = b' ';    // Space

    /// Holds the result of a parsing function.
    ///
    /// It takes a byte slice as input and uses `nom::Err<()>` as error
    /// variant. The type only depends the output type `O`.
    pub type ParseResult<'a, O> = Result<(&'a [u8], O), nom::Err<()>>;

    // pub use super::field::parse_field_ref;
    // pub use super::occurrence::parse_occurrence_ref;
    // pub use super::occurrence::parse_occurrence_digits;
    pub use super::subfield::parse_subfield_code;
    pub use super::subfield::parse_subfield_value;
    pub use super::tag::parse_tag;
}
