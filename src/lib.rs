//! This library provides types to work with bibliographic records
//! encoded in (normalized) PICA+, the internal data format of the
//! [OCLC](https://www.oclc.org) cataloging system.

pub use error::Error;
pub use record::{ByteRecord, StringRecord};

mod error;
mod fmt;
pub mod matcher;
mod parser;
pub mod path;
pub mod prelude;
pub mod primitives;
pub mod query;

pub mod io {
    pub use super::reader::{
        ReadPicaError, ReaderBuilder, RecordsIter,
    };
    pub use super::writer::{
        ByteRecordWrite, GzipWriter, PlainWriter, WriterBuilder,
    };
}

mod reader;
mod record;
mod writer;
