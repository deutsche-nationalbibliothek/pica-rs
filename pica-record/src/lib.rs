mod arbitrary;
mod error;
mod subfield;

pub use error::ParsePicaError;
pub use subfield::{Subfield, SubfieldRef};

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
}
