//! This library provides types to work with bibliographic records
//! encoded in (normalized) PICA+, the internal data format of the
//! [OCLC](https://www.oclc.org) cataloging system.

pub use error::Error;
pub use record::{ByteRecord, StringRecord};

mod error;
#[cfg(feature = "unstable")]
mod fmt;
pub mod matcher;
mod parser;
pub mod path;
pub mod prelude;
pub mod primitives;
mod record;
