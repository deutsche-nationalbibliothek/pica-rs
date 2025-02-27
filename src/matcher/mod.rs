//! Various matcher against record primitives.

pub use builder::RecordMatcherBuilder;
pub use error::ParseMatcherError;
pub use occurrence::OccurrenceMatcher;
pub use operator::{BooleanOp, RelationalOp};
pub use options::MatcherOptions;
pub use quantifier::Quantifier;
pub use record::RecordMatcher;
pub use tag::TagMatcher;

mod builder;
mod error;
pub mod field;
pub(crate) mod occurrence;
mod operator;
mod options;
mod quantifier;
mod record;
pub mod subfield;
pub(crate) mod tag;
