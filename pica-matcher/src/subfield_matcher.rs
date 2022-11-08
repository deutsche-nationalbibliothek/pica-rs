use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{all_consuming, map, value};
use nom::multi::many1;
use nom::sequence::{delimited, terminated};
use nom::Finish;
use pica_record::parser::{parse_subfield_code, ParseResult};
use pica_record::Subfield;

use crate::common::ws;
use crate::{MatcherOptions, ParseMatcherError};

const SUBFIELD_CODES: &str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub trait Matcher: Sized {
    /// Create a new matcher from a string slice.
    fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(Self::parse)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }

    /// Parse the matcher from a byte slice.
    fn parse(i: &[u8]) -> ParseResult<Self>;

    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>>,
        options: &MatcherOptions,
    ) -> bool;
}

/// A matcher that checks the existance of subfield.
///
/// This matcher can be used to determine if a single subfield or a list
/// of subfields contains at least one subfield with a code, that is
/// contained in the matcher's code list.
#[derive(Debug, PartialEq, Eq)]
pub struct ExistsMatcher {
    codes: Vec<char>,
}

impl Matcher for ExistsMatcher {
    /// Returns true if at least one subfield is found with a code which
    /// is in the matcher's code list.
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>>,
        _options: &MatcherOptions,
    ) -> bool {
        subfields
            .into_iter()
            .any(|subfield| self.codes.contains(&subfield.code()))
    }

    /// Parse the matcher expression from a byte slice.
    ///
    /// # Grammar
    ///
    /// ```txt
    /// exists-matcher ::= subfield-codes ws* '?'
    /// subfield-codes ::= subfield-code-list1 | subfield-code-list2 |
    ///                    subfield-code-wildcard | subfield-code |
    /// subfield-code-list1 ::= '[' subfield-code+ ']'
    /// subfield-code-list2 ::= subfield-code+
    /// subfield-code-wildcard ::= '*'
    /// subfield-code ::= [A-Z] | [a-z] | [0-9]
    /// ```
    fn parse(i: &[u8]) -> ParseResult<Self> {
        map(terminated(ws(parse_subfield_codes), char('?')), |codes| {
            Self { codes }
        })(i)
    }
}

/// Parse a list of subfield codes.
fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        delimited(char('['), many1(parse_subfield_code), char(']')),
        many1(parse_subfield_code),
        map(parse_subfield_code, |code| vec![code]),
        value(SUBFIELD_CODES.chars().collect(), char('*')),
    ))(i)
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_parse_subfield_codes() {
        assert_finished_and_eq!(parse_subfield_codes(b"a"), vec!['a']);
        assert_finished_and_eq!(
            parse_subfield_codes(b"[12]"),
            vec!['1', '2']
        );
        assert_finished_and_eq!(
            parse_subfield_codes(b"012"),
            vec!['0', '1', '2']
        );
        assert_finished_and_eq!(
            parse_subfield_codes(b"*"),
            SUBFIELD_CODES.chars().collect::<Vec<char>>()
        );

        assert_error!(parse_subfield_codes(b"!"));
        assert_error!(parse_subfield_codes(b"[a1!]"));
    }
}
