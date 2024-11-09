use std::fmt::{self, Display};

use bstr::{ByteSlice, ByteVec};
use winnow::combinator::{alt, separated};
use winnow::{PResult, Parser};

#[cfg(feature = "unstable")]
use crate::fmt::Format;
use crate::parser::{parse_string, ws};
use crate::path::{parse_path, Path};

/// An error that can occur when parsing a query expression.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("{0}")]
pub struct ParseQueryError(pub(crate) String);

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    fragments: Vec<Fragment>,
    raw_query: String,
}

impl Query {
    /// Creates a new [Query].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// query expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let _query = Query::new("003@.0,002@.0")?;
    /// let _query = Query::new("'x', 002@.0")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(query: &str) -> Result<Self, ParseQueryError> {
        parse_query.parse(query.as_bytes()).map_err(|_| {
            ParseQueryError(format!("invalid query '{query}'"))
        })
    }
}

fn parse_query(i: &mut &[u8]) -> PResult<Query> {
    separated(1.., parse_fragment, ws(','))
        .with_taken()
        .map(|(fragments, raw_query)| {
            let raw_query = raw_query.to_str().unwrap().to_string();
            Query {
                fragments,
                raw_query,
            }
        })
        .parse_next(i)
}

impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_query)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Query {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Query {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Fragment {
    Path(Path),
    Literal(String),
    #[cfg(feature = "unstable")]
    Format(Format),
}

fn parse_fragment(i: &mut &[u8]) -> PResult<Fragment> {
    alt((
        parse_path.map(Fragment::Path),
        #[cfg(feature = "unstable")]
        parse_format.map(Fragment::Format),
        parse_string
            .map(|s| s.into_string().unwrap())
            .map(Fragment::Literal),
    ))
    .parse_next(i)
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    #[cfg(feature = "serde")]
    fn test_path_serde() -> TestResult {
        assert_tokens(&Query::new("003@.0")?, &[Token::Str("003@.0")]);
        assert_tokens(&Query::new("'foo'")?, &[Token::Str("'foo'")]);
        assert_tokens(
            &Query::new("'x', 002@.0")?,
            &[Token::Str("'x', 002@.0")],
        );
        assert_tokens(
            &Query::new("041A/*{ 9 | 9? }")?,
            &[Token::Str("041A/*{ 9 | 9? }")],
        );
        Ok(())
    }
}
