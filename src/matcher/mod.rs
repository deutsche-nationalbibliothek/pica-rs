//! Various matcher against record primitives.

pub use error::ParseMatcherError;
pub use occurrence::OccurrenceMatcher;
pub use operator::{BooleanOp, RelationalOp};
pub use options::MatcherOptions;
pub use quantifier::Quantifier;
pub use record::RecordMatcher;
pub use tag::TagMatcher;

mod error;
pub mod field;
mod occurrence;
mod operator;
mod options;
mod quantifier;
mod record;
pub mod subfield;
mod tag;
