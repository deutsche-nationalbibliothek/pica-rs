//! This library provides types to work with bibliographic records
//! encoded in (normalized) PICA+, the internal data format of the
//! [OCLC](https://www.oclc.org) cataloging system.

pub use error::{Error, ParsePicaError};
pub use record::{
    ByteRecord, Field, FieldRef, Level, Occurrence, OccurrenceRef,
    Record, RecordRef, StringRecord, Subfield, SubfieldCode,
    SubfieldRef, SubfieldValue, SubfieldValueRef, Tag, TagRef,
};

mod error;
mod record;
