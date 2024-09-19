//! Various matcher against record primitives.

pub use error::ParseMatcherError;
pub use occurrence::OccurrenceMatcher;
pub use operator::{BooleanOp, RelationalOp};
pub use options::MatcherOptions;
pub use quantifier::Quantifier;
pub use tag::TagMatcher;

mod error;
mod occurrence;
mod operator;
mod options;
mod quantifier;
pub mod subfield;
mod tag;
