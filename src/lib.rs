#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub use self::error::{Error, Result};
pub use self::field::Field;
pub use self::matcher_old::{
    ComparisonOp, FieldMatcher, MatcherFlags, RecordMatcher,
    SubfieldListMatcher, SubfieldMatcher,
};
pub use self::occurrence::Occurrence;
pub use self::parser::{ParsePathError, ParsePicaError};
pub use self::path::Path;
pub use self::reader::{Reader, ReaderBuilder};
pub use self::record::{ByteRecord, StringRecord};
pub use self::select::{Outcome, Selector, Selectors};
pub use self::subfield::Subfield;
pub use self::tag::{Level, Tag};
pub use self::writer::{GzipWriter, PicaWriter, PlainWriter, WriterBuilder};

mod common;
mod error;
mod field;
pub mod matcher;
mod matcher_old;
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
