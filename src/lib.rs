#[macro_use]
extern crate lazy_static;

pub use self::error::{Error, Result};
pub use self::filter::{Filter, ParseFilterError};
pub use self::occurrence::{Occurrence, OccurrenceMatcher};
pub use self::parser::{ParsePathError, ParsePicaError};
pub use self::path::Path;
pub use self::reader::{Reader, ReaderBuilder};
pub use self::record::{ByteRecord, Field, StringRecord, Subfield};
pub use self::select::{Outcome, Selector, Selectors};
pub use self::tag::{Tag, TagMatcher};
pub use self::writer::{GzipWriter, PicaWriter, PlainWriter, WriterBuilder};

mod error;
mod filter;
mod occurrence;
mod parser;
mod path;
mod reader;
mod record;
mod select;
mod tag;
mod writer;
