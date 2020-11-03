extern crate nom;

pub mod error;
mod field;
pub mod parser;
mod path;
mod record;
mod subfield;

pub use field::Field;
pub use path::Path;

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
