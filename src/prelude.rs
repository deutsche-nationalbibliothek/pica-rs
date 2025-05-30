//! General-use types and traits from this crate.
//!
//! This module contains the most used types, type aliases, traits,
//! functions and macros for glob import.
//!
//! # Example
//!
//! ```rust
//! use pica_record::prelude::*;
//!
//! let record = ByteRecord::from_bytes(b"002@ \x1f0Olfo\x1e\n")?;
//! assert!(record.validate().is_ok());
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub use crate::fmt::{Format, FormatExt, FormatOptions};
pub use crate::matcher::field::FieldMatcher;
pub use crate::matcher::subfield::SubfieldMatcher;
pub use crate::matcher::{
    MatcherOptions, OccurrenceMatcher, RecordMatcher,
    RecordMatcherBuilder, TagMatcher,
};
pub use crate::path::{Path, PathExt};
pub use crate::query::{Query, QueryExt, QueryOptions};
pub use crate::reader::{ReadPicaError, ReaderBuilder, RecordsIter};
pub use crate::writer::{ByteRecordWrite, WriterBuilder};
pub use crate::{ByteRecord, Error, StringRecord};
