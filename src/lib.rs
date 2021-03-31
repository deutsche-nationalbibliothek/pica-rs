//! `pica-rs` provides a library and tools to work with bibliographic records
//! encoded in PICA+.
//!
//! # Thanks
//!
//! Most of the architectural decisions and concepts are based on the [CSV
//! toolkit xsv](https://github.com/BurntSushi/xsv) and [CSV parser
//! library](https://github.com/BurntSushi/rust-csv) written by Andrew Gallant.

pub use error::{Error, Result};
pub use record::{ByteRecord, Field, Occurrence, ParsePicaError, Subfield};

mod error;
mod record;
