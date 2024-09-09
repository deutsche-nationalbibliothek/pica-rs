//! This library provides types to work with bibliographic records
//! encoded in (normalized) PICA+, the internal data format of the
//! [OCLC](https://www.oclc.org) catalog system.

pub use error::{Error, ParsePicaError};
pub use record::{SubfieldCode, SubfieldValue, SubfieldValueRef};

mod error;
mod record;
