//! This crate provides the low-level primitives to work with
//! bibliographic records encoded in PICA+. There exists a read-only
//! (immutable) and mutable variant of each primitive.

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod arbitrary;
mod error;
mod subfield;
mod tag;

pub use error::ParsePicaError;
pub use subfield::{Subfield, SubfieldRef};
pub use tag::{Tag, TagRef};

/// Parsers recognizing low-level primitives (e.g. subfield codes).
pub mod parser {
    pub(crate) const RS: u8 = b'\x1E'; // Record Separator
    pub(crate) const US: u8 = b'\x1F'; // Unit Separator

    /// Holds the result of a parsing function.
    ///
    /// It takes a byte slice as input and uses `nom::Err<()>` as error
    /// variant. The type only depends the output type `O`.
    pub type ParseResult<'a, O> = Result<(&'a [u8], O), nom::Err<()>>;

    pub use super::subfield::{
        parse_subfield_code, parse_subfield_value,
    };
    pub use super::tag::parse_tag_ref;
}
