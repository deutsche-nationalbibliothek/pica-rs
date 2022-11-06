use std::fmt::Display;

use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{all_consuming, map, value};
use nom::multi::many1;
use nom::sequence::delimited;
use nom::Finish;
use pica_record::parser::{parse_subfield_code, ParseResult};

use crate::ParseMatcherError;

const SUBFILED_CODES: &'static str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// A matcher that matches against PICA+
/// [Subfield](`pica_record::Subfield`).
#[derive(Debug)]
pub struct SubfieldMatcher {
    kind: SubfieldMatcherKind,
    matcher_str: String,
}

#[derive(Debug, PartialEq, Eq)]
enum SubfieldMatcherKind {
    None,
}
impl SubfieldMatcher {
    /// Create a new subfield matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::TagMatcher;
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = TagMatcher::new("003@")?;
    ///     assert_eq!(matcher, TagRef::new("003@"));
    ///
    ///     # assert!(TagMatcher::new("003!").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T>(expr: T) -> Result<Self, ParseMatcherError>
    where
        T: AsRef<[u8]> + Display,
    {
        all_consuming(parse_subfield_matcher_kind)(expr.as_ref())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(
                    expr.to_string(),
                )
            })
            .map(|(_, kind)| Self {
                matcher_str: expr.to_string(),
                kind,
            })
    }
}

fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        delimited(char('['), many1(parse_subfield_code), char(']')),
        many1(parse_subfield_code),
        map(parse_subfield_code, |code| vec![code]),
        value(SUBFILED_CODES.chars().collect(), char('*')),
    ))(i)
}

fn parse_subfield_matcher_kind(
    _i: &[u8],
) -> ParseResult<SubfieldMatcherKind> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_parse_subfield_codes() {
        let expected = vec!['0'];
        assert_done_and_eq!(parse_subfield_codes(b"0"), expected);

        let expected = vec!['0', '1', '2'];
        assert_done_and_eq!(parse_subfield_codes(b"[012]"), expected);

        let expected = vec!['0', '1', '2'];
        assert_done_and_eq!(parse_subfield_codes(b"012"), expected);

        let expected = SUBFILED_CODES.chars().collect::<Vec<char>>();
        assert_done_and_eq!(parse_subfield_codes(b"*"), expected);

        assert_error!(parse_subfield_codes(b"!"));
        assert_error!(parse_subfield_codes(b"[012!]"));
    }
}
