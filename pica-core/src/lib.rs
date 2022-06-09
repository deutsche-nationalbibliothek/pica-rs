mod error;
mod tag;
mod types;

pub use error::ParseError;
pub use tag::{Tag, TagRef};
pub use types::ParseResult;

pub mod parser {
    pub use super::tag::parse_tag;
}
