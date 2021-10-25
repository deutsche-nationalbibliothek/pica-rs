#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub use self::error::{Error, Result};
pub use self::field::{Field, FieldMatcher};
pub use self::filter::{Filter, ParseFilterError};
pub use self::occurrence::{Occurrence, OccurrenceMatcher};
pub use self::parser::{ParsePathError, ParsePicaError};
pub use self::path::Path;
pub use self::reader::{Reader, ReaderBuilder};
pub use self::record::{ByteRecord, StringRecord};
pub use self::select::{Outcome, Selector, Selectors};
pub use self::subfield::{Subfield, SubfieldMatcher, SubfieldsMatcher};
pub use self::tag::{Level, Tag, TagMatcher};
pub use self::writer::{GzipWriter, PicaWriter, PlainWriter, WriterBuilder};

mod common;
mod error;
mod field;
mod filter;
mod occurrence;
mod parser;
mod path;
mod reader;
mod record;
mod select;
mod subfield;
mod tag;
#[cfg(test)]
mod test;
mod writer;
