//! `pica-rs` provides a library and tools to work with bibliographic records
//! encoded in PICA+.

#[macro_use]
extern crate lazy_static;

pub use error::{Error, Result};
pub use path::{OccurrenceMatcher, ParsePathError, Path};
pub use reader::{Reader, ReaderBuilder};
pub use record::{
    ByteRecord, Field, Occurrence, ParsePicaError, StringRecord, Subfield,
};
pub use writer::{Writer, WriterBuilder};

mod error;
mod path;
mod reader;
mod record;
mod writer;
