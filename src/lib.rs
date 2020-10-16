extern crate nom;

pub mod error;
mod field;
pub mod parser;
mod record;
mod subfield;

pub use field::Field;

/// Pica+ record.
///
/// ```
/// use pica::Record;
///
/// let record = Record { fields: vec![] };
/// assert!(record.fields.is_empty());
/// ```
pub use record::Record;

pub use subfield::Subfield;
