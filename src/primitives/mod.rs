//! Low-level primitives to work with (normalized) PICA+ records.

pub use error::ParsePicaError;
pub use field::{Field, FieldRef};
pub use occurrence::{Occurrence, OccurrenceRef};
pub use record::{Record, RecordRef};
pub use subfield::{
    Subfield, SubfieldCode, SubfieldRef, SubfieldValue,
    SubfieldValueRef,
};
pub use tag::{Level, Tag, TagRef};

mod error;
mod field;
mod occurrence;
pub(crate) mod parse;
mod record;
mod subfield;
mod tag;
