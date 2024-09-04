//! This crate provides the low-level primitives to work with
//! bibliographic records encoded in PICA+.
//!
//! There exists an immutable and a mutable variant for each primitive.
//! The immutable variant is used to parse the corresponding component
//! of a record, without owning the data. This type is mostly a wrapper
//! of the underlying data (byte slices). On the other hand there is
//! also a mutable variant, which is used in upstream crates. This
//! variant owns it's data.
//!
//! This crate also provides two higher-level data structure to work
//! with records: [`ByteRecord`] and [`StringRecord`]. The first type
//! is a wrapper of a [`RecordRef`] and provides more functions to work
//! with records as well as an mechanism to cache the complete byte
//! sequence of the original record. This improves the performance when
//! the record is written back to a stream. It is important to note that
//! a [`ByteRecord`] might contain invalid UTF-8 data. When it's
//! important to guarantee valid UTF-8 data, use a [`StringRecord`]
//! instead.
//!
//! Finally, the [`io`] module provides utilities for reading and
//! writing PICA+ records and the [`parser`] module exposes the internal
//! parser combinators, which are used in upstream crates (matcher,
//! select).

pub use error::PicaError;
pub use primitives::{SubfieldCode, SubfieldValue, SubfieldValueRef};

/// Parsers recognizing low-level primitives (e.g. subfield codes).
#[rustfmt::skip]
pub mod parser {
    pub use super::primitives::parse_subfield_code;
    pub use super::primitives::parse_subfield_value_ref;

    // TODO
    pub use super::occurrence::parse_occurrence_digits;
    pub use super::tag::parse_tag;
}

mod primitives;

// -----{ TODO }-----------------------------------------

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
pub use level::{Level, ParseLevelError};
pub use occurrence::{Occurrence, OccurrenceRef};
pub use record::{ByteRecord, Record, RecordRef, StringRecord};
pub use subfield::{Subfield, SubfieldRef};
pub use tag::{Tag, TagRef};
