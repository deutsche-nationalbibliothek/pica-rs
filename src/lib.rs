#[macro_use]
extern crate lazy_static;

pub use self::error::{Error, Result};
pub use self::filter::{Filter, OccurrenceMatcher, ParseFilterError};
pub use self::path::Path;
pub use self::record::{ByteRecord, Field, Occurrence, StringRecord, Subfield};
pub use self::select::{Outcome, Selector, Selectors};

mod error;
mod filter;
mod parser;
mod path;
mod record;
mod select;
