extern crate nom;
extern crate serde;
extern crate serde_json;

pub use self::error::PicaParseError;
pub use self::field::{parse_field, Field};
pub use self::subfield::{parse_subfield, Subfield};

mod error;
mod field;
mod subfield;

// mod filter;
// pub mod parser;
// mod path;
// mod record;

// pub use field::Field;
// pub use filter::Filter;
// pub use path::Path;
// pub use record::Record;
