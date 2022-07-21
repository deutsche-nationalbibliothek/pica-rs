mod error;
mod tag_matcher;

pub use error::ParseError;
pub use tag_matcher::TagMatcher;

/// Parser combinator for parsing matchers.
pub mod parser {
    pub use crate::tag_matcher::parse_tag_matcher;
}

/// Holds the result of a test function.
#[cfg(test)]
pub(crate) type TestResult = Result<(), Box<dyn std::error::Error>>;
