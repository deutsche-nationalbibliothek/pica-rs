//! This library provides types to work with bibliographic records
//! encoded in (normalized) PICA+, the internal data format of the
//! [OCLC](https://www.oclc.org) catalog system.

pub use error::{Error, ParsePicaError};
pub use record::{
    Level, Occurrence, OccurrenceRef, Subfield, SubfieldCode,
    SubfieldRef, SubfieldValue, SubfieldValueRef, Tag, TagRef,
};

mod error;
mod record;
