//! This module contains a bunch of matcher, to test
//! [Subfields](pica_record::Subfield).
use bstr::{BString, ByteSlice};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, map, value, verify};
use nom::multi::many1;
use nom::sequence::{delimited, terminated, tuple};
use nom::Finish;
use pica_record::parser::{parse_subfield_code, ParseResult};
use pica_record::Subfield;
use regex::bytes::RegexBuilder;
use regex::Regex;
use strsim::normalized_levenshtein;

use crate::common::{
    parse_relational_op_str, parse_string, ws, RelationalOp,
};
use crate::{MatcherOptions, ParseMatcherError};

const SUBFIELD_CODES: &str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// A trait that provides the basic matcher API.
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

/// A matcher that tests for the existance of subfield.
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

fn case_compare(lhs: &[u8], rhs: &[u8], case_ignore: bool) -> bool {
    if case_ignore {
        lhs.to_lowercase() == rhs.to_lowercase()
    } else {
        lhs == rhs
    }
}

/// A matcher that tests a relations between values.
///
/// This matcher provides the following relational operators:
/// * Equal (`==`)
/// * Not Equal (`!=`)
/// * StartsWith (`=^`)
/// * EndsWith (`=$`)
/// * Similar (`=*`)
#[derive(Debug, PartialEq, Eq)]
pub struct RelationMatcher {
    codes: Vec<char>,
    op: RelationalOp,
    value: BString,
}

impl Matcher for RelationMatcher {
    /// Returns true if at least one subfield is found, when the
    /// subfield's value and the matcher value are related. The two
    /// values are related iff the relation defined by the operator
    /// exists.
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>>,
        options: &MatcherOptions,
    ) -> bool {
        for subfield in subfields {
            if !self.codes.contains(&subfield.code()) {
                continue;
            }

            let value = subfield.value().as_ref();
            let result = match self.op {
                RelationalOp::Eq => case_compare(
                    &self.value,
                    value,
                    options.case_ignore,
                ),
                RelationalOp::Ne => !case_compare(
                    &self.value,
                    value,
                    options.case_ignore,
                ),
                RelationalOp::StartsWith => {
                    if options.case_ignore {
                        value
                            .to_lowercase()
                            .starts_with(&self.value.to_lowercase())
                    } else {
                        value.starts_with(&self.value)
                    }
                }
                RelationalOp::EndsWith => {
                    if options.case_ignore {
                        value
                            .to_lowercase()
                            .ends_with(&self.value.to_lowercase())
                    } else {
                        value.ends_with(&self.value)
                    }
                }
                RelationalOp::Similar => {
                    let score = if options.case_ignore {
                        normalized_levenshtein(
                            &self.value.to_string().to_lowercase(),
                            &value.to_str_lossy().to_lowercase(),
                        )
                    } else {
                        normalized_levenshtein(
                            &self.value.to_string(),
                            &value.to_str_lossy(),
                        )
                    };

                    score > options.strsim_threshold
                }
            };

            if result {
                return true;
            }
        }

        false
    }

    /// Parse a relational expression
    fn parse(i: &[u8]) -> ParseResult<RelationMatcher> {
        map(
            tuple((
                ws(parse_subfield_codes),
                ws(parse_relational_op_str),
                map(ws(parse_string), BString::from),
            )),
            |(codes, op, value)| RelationMatcher { codes, op, value },
        )(i)
    }
}

/// A matcher that tests against a regular expression.
#[derive(Debug, PartialEq, Eq)]
pub struct RegexMatcher {
    codes: Vec<char>,
    pattern: String,
    invert: bool,
}

impl Matcher for RegexMatcher {
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>>,
        options: &MatcherOptions,
    ) -> bool {
        let re = RegexBuilder::new(&self.pattern)
            .case_insensitive(options.case_ignore)
            .build()
            .unwrap();

        for subfield in subfields {
            if !self.codes.contains(&subfield.code()) {
                continue;
            }

            let mut result = re.is_match(subfield.value().as_ref());
            if self.invert {
                result = !result;
            }

            if result {
                return true;
            }
        }

        false
    }

    fn parse(i: &[u8]) -> ParseResult<Self> {
        map(
            tuple((
                parse_subfield_codes,
                alt((
                    value(false, ws(tag("=~"))),
                    value(true, ws(tag("!~"))),
                )),
                verify(parse_string, |x| Regex::new(x).is_ok()),
            )),
            |(codes, invert, pattern)| RegexMatcher {
                codes,
                pattern,
                invert,
            },
        )(i)
    }
}

/// Parse a list of subfield codes
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
