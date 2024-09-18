//! Various matcher against records (and record primitives).

pub use error::ParseMatcherError;
pub use operator::{BooleanOp, RelationalOp};
pub use options::MatcherOptions;
pub use quantifier::Quantifier;
pub use subfield::{
    CardinalityMatcher, ExistsMatcher, InMatcher, RegexMatcher,
    RegexSetMatcher, RelationMatcher, SingletonMatcher,
    SubfieldMatcher,
};

mod error;
mod operator;
mod options;
mod parse;
mod quantifier;
mod string;
mod subfield;
