//! Matcher that works on PICA+ [Subfields](pica_record::Subfield).

use std::ops::{BitAnd, BitOr};

use bstr::{BString, ByteSlice};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{
    all_consuming, cut, map, map_res, opt, value, verify,
};
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::Finish;
use pica_record::parser::{parse_subfield_code, ParseResult};
use pica_record::Subfield;
use regex::bytes::RegexBuilder;
use regex::Regex;
use strsim::normalized_levenshtein;

use crate::common::{
    parse_relational_op_str, parse_relational_op_usize, parse_string,
    ws, BooleanOp, RelationalOp,
};
use crate::{MatcherOptions, ParseMatcherError};

const SUBFIELD_CODES: &str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Parse a list of subfield codes
fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        delimited(char('['), many1(parse_subfield_code), char(']')),
        map(parse_subfield_code, |code| vec![code]),
        value(SUBFIELD_CODES.chars().collect(), char('*')),
    ))(i)
}

/// A trait that provides the basic matcher API.
pub trait Matcher {
    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>> + Clone,
        options: &MatcherOptions,
    ) -> bool;
}

/// A matcher that checks if a subfield exists.
///
/// This matcher can be used to determine if a single subfield or a list
/// of subfields contains at least one subfield with a code, that is
/// contained in the matcher's code list.
#[derive(Debug, PartialEq, Eq)]
pub struct ExistsMatcher {
    codes: Vec<char>,
}

/// Parse the matcher expression from a byte slice.
///
/// # Grammar
///
/// ```txt
/// exists-matcher ::= subfield-codes ws* '?'
/// subfield-codes ::= subfield-code-list1
///                  | subfield-code-wildcard
///                  | subfield-code
/// subfield-code-list1 ::= '[' subfield-code+ ']'
/// subfield-code-wildcard ::= '*'
/// subfield-code ::= [A-Z] | [a-z] | [0-9]
/// ```
pub(crate) fn parse_exists_matcher(
    i: &[u8],
) -> ParseResult<ExistsMatcher> {
    map(terminated(ws(parse_subfield_codes), char('?')), |codes| {
        ExistsMatcher { codes }
    })(i)
}

impl ExistsMatcher {
    /// Create a new exists matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::{ExistsMatcher, Matcher};
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = ExistsMatcher::new("0?")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &SubfieldRef::new('0', "123456789X"),
    ///         &Default::default()
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &SubfieldRef::new('a', "abc"),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_exists_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }
}

impl Matcher for ExistsMatcher {
    /// Returns `true` if at least one subfield is found with a code
    /// which is in the matcher's code list.
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>> + Clone,
        _options: &MatcherOptions,
    ) -> bool {
        subfields
            .into_iter()
            .any(|subfield| self.codes.contains(&subfield.code()))
    }
}

/// A matcher that checks relations between (string) values.
///
/// This matcher provides basic relational operations between string
/// values; the following operators
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

/// Parse a relational expression
fn parse_relation_matcher(i: &[u8]) -> ParseResult<RelationMatcher> {
    map(
        tuple((
            ws(parse_subfield_codes),
            ws(parse_relational_op_str),
            map(ws(parse_string), BString::from),
        )),
        |(codes, op, value)| RelationMatcher { codes, op, value },
    )(i)
}

impl RelationMatcher {
    /// Create a new relation matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::{Matcher, RelationMatcher};
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = RelationMatcher::new("0 == '123456789X'")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &SubfieldRef::new('0', "123456789X"),
    ///         &Default::default()
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &SubfieldRef::new('0', "123456789!"),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_relation_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }

    /// Returns `true` if the given value is equal to the matcher's
    /// value. If the `case_ignore` flag is set, both strings will be
    /// converted to lowercase first.
    fn compare(&self, value: &[u8], options: &MatcherOptions) -> bool {
        if options.case_ignore {
            self.value.to_lowercase() == value.to_lowercase()
        } else {
            self.value == value
        }
    }

    /// Returns `true` if the given values is a prefix of the matcher's
    /// value, otherwise `false`. If the `case_ignore` flag ist set,
    /// both strings will be converted to lowercase first.
    fn starts_with(
        &self,
        value: &[u8],
        options: &MatcherOptions,
        invert: bool,
    ) -> bool {
        let mut result = if options.case_ignore {
            value.to_lowercase().starts_with(&self.value.to_lowercase())
        } else {
            value.starts_with(&self.value)
        };

        if invert {
            result = !result
        }

        result
    }

    /// Returns `true` if the given values is a suffix of the matcher's
    /// value, otherwise `false`. If the `case_ignore` flag ist set,
    /// both strings will be converted to lowercase first.
    fn ends_with(
        &self,
        value: &[u8],
        options: &MatcherOptions,
        invert: bool,
    ) -> bool {
        let mut result = if options.case_ignore {
            value.to_lowercase().ends_with(&self.value.to_lowercase())
        } else {
            value.ends_with(&self.value)
        };

        if invert {
            result = !result;
        }

        result
    }

    /// Returns `true` if the given value is similar to the matcher's
    /// value. The similarity score is determined by calculating the
    /// normalized levenshtein distance between both strings. If the
    /// `case_ignore` flag is set, both strings will be converted to
    /// lowercase first.
    fn is_similar(
        &self,
        value: &[u8],
        options: &MatcherOptions,
    ) -> bool {
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
}

impl Matcher for RelationMatcher {
    /// Returns true if at least one subfield is found, when the
    /// subfield's value and the matcher value are related. The two
    /// values are related iff the relation defined by the operator
    /// exists.
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        subfields
            .into_iter()
            .filter(|s| self.codes.contains(&s.code()))
            .any(|subfield| {
                let value = subfield.value().as_ref();
                match self.op {
                    RelationalOp::Eq => self.compare(value, options),
                    RelationalOp::Ne => !self.compare(value, options),
                    RelationalOp::StartsWith => {
                        self.starts_with(value, options, false)
                    }
                    RelationalOp::StartsNotWith => {
                        self.starts_with(value, options, true)
                    }
                    RelationalOp::EndsWith => {
                        self.ends_with(value, options, false)
                    }
                    RelationalOp::EndsNotWith => {
                        self.ends_with(value, options, true)
                    }
                    RelationalOp::Similar => {
                        self.is_similar(value, options)
                    }
                    _ => unreachable!(),
                }
            })
    }
}

/// A matcher that checks a subfield value against a regex.
#[derive(Debug, PartialEq, Eq)]
pub struct RegexMatcher {
    codes: Vec<char>,
    pattern: String,
    invert: bool,
}

/// Parse a regex matcher expression
fn parse_regex_matcher(i: &[u8]) -> ParseResult<RegexMatcher> {
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

impl RegexMatcher {
    /// Create a new regex matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::{Matcher, RegexMatcher};
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = RegexMatcher::new("0 =~ '^Oa'")?;
    ///     assert!(matcher.is_match(
    ///         &SubfieldRef::new('0', "Oa"),
    ///         &Default::default()
    ///     ));
    ///
    ///     let matcher = RegexMatcher::new("0 !~ '^Oa'")?;
    ///     assert!(matcher.is_match(
    ///         &SubfieldRef::new('0', "Ob"),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_regex_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }
}

impl Matcher for RegexMatcher {
    /// Returns true if at least one subfield value is found, that
    /// matches against the regular expression.
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        let re = RegexBuilder::new(&self.pattern)
            .case_insensitive(options.case_ignore)
            .build()
            .unwrap();

        subfields
            .into_iter()
            .filter(|s| self.codes.contains(&s.code()))
            .any(|subfield| {
                let mut result = re.is_match(subfield.value().as_ref());
                if self.invert {
                    result = !result;
                }

                result
            })
    }
}

/// A matcher that checks if a subfield value is in a predefined list.
#[derive(Debug, PartialEq, Eq)]
pub struct InMatcher {
    codes: Vec<char>,
    values: Vec<BString>,
    invert: bool,
}

/// Parse a in matcher expression.
fn parse_in_matcher(i: &[u8]) -> ParseResult<InMatcher> {
    map(
        tuple((
            parse_subfield_codes,
            map(opt(ws(tag("not"))), |not| not.is_some()),
            ws(tag("in")),
            delimited(
                ws(char('[')),
                separated_list1(
                    ws(char(',')),
                    map(parse_string, BString::from),
                ),
                ws(char(']')),
            ),
        )),
        |(codes, invert, _, values)| InMatcher {
            codes,
            values,
            invert,
        },
    )(i)
}

impl InMatcher {
    /// Create a new matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::{InMatcher, Matcher};
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = InMatcher::new("0 in ['abc', 'def']")?;
    ///     assert!(matcher.is_match(
    ///         &SubfieldRef::new('0', "def"),
    ///         &Default::default()
    ///     ));
    ///
    ///     let matcher = InMatcher::new("0 not in ['abc', 'def']")?;
    ///     assert!(matcher.is_match(
    ///         &SubfieldRef::new('0', "hij"),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_in_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }
}

impl Matcher for InMatcher {
    /// Returns `true` if at least one subfield is found, where the
    /// value is contained in the matcher list.
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        subfields
            .into_iter()
            .filter(|s| self.codes.contains(&s.code()))
            .any(|subfield| {
                let mut result =
                    self.values.iter().any(|value: &BString| {
                        if options.case_ignore {
                            subfield.value().as_ref().to_lowercase()
                                == value.to_lowercase()
                        } else {
                            subfield.value().as_ref() == value
                        }
                    });

                if self.invert {
                    result = !result;
                }

                result
            })
    }
}

/// A matcher that checks the number of occurrences of a subfield.
#[derive(Debug, PartialEq, Eq)]
pub struct CardinalityMatcher {
    code: char,
    op: RelationalOp,
    value: usize,
}

/// Parse a cardinality matcher expression.
fn parse_cardinality_matcher(
    i: &[u8],
) -> ParseResult<CardinalityMatcher> {
    map(
        preceded(
            ws(char('#')),
            cut(tuple((
                ws(parse_subfield_code),
                ws(parse_relational_op_usize),
                map_res(digit1, |s| {
                    std::str::from_utf8(s).unwrap().parse::<usize>()
                }),
            ))),
        ),
        |(code, op, value)| CardinalityMatcher { code, op, value },
    )(i)
}

impl CardinalityMatcher {
    /// Create a new matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::{CardinalityMatcher, Matcher};
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = CardinalityMatcher::new("#0 > 1")?;
    ///
    ///     assert!(matcher.is_match(
    ///         vec![
    ///             &SubfieldRef::new('0', "def"),
    ///             &SubfieldRef::new('0', "abc")
    ///         ],
    ///         &Default::default()
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &SubfieldRef::new('0', "def"),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_cardinality_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }
}

impl Matcher for CardinalityMatcher {
    /// Returns true of number of fields with a code equal to the
    /// matcher's code is `==`, `!=`, `>=`, `>`, `<=`, or `<` than the
    /// matcher's value.
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>> + Clone,
        _options: &MatcherOptions,
    ) -> bool {
        let count = subfields
            .into_iter()
            .filter(|&s| self.code == s.code())
            .count();

        match self.op {
            RelationalOp::Eq => count == self.value,
            RelationalOp::Ne => count != self.value,
            RelationalOp::Ge => count >= self.value,
            RelationalOp::Gt => count > self.value,
            RelationalOp::Le => count <= self.value,
            RelationalOp::Lt => count < self.value,
            _ => unreachable!(),
        }
    }
}

/// A matcher that checks for the singleton matcher.
///
/// This matcher combines all atomic, singleton matcher into a new
/// matcher.
#[derive(Debug, PartialEq, Eq)]
pub enum SingletonMatcher {
    Cardinality(CardinalityMatcher),
    Exists(ExistsMatcher),
    In(InMatcher),
    Regex(RegexMatcher),
    Relation(RelationMatcher),
}

/// Parse a singleton matcher expression.
fn parse_singleton_matcher(i: &[u8]) -> ParseResult<SingletonMatcher> {
    alt((
        map(parse_cardinality_matcher, SingletonMatcher::Cardinality),
        map(parse_exists_matcher, SingletonMatcher::Exists),
        map(parse_in_matcher, SingletonMatcher::In),
        map(parse_regex_matcher, SingletonMatcher::Regex),
        map(parse_relation_matcher, SingletonMatcher::Relation),
    ))(i)
}

impl SingletonMatcher {
    /// Create a new singleton matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::{Matcher, SingletonMatcher};
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SingletonMatcher::new("0 != '123456789X'")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &SubfieldRef::new('0', "2345678901"),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_singleton_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }
}

impl Matcher for SingletonMatcher {
    /// Returns `true` if the underlying matcher returns `true`.
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Cardinality(m) => m.is_match(subfields, options),
            Self::Exists(m) => m.is_match(subfields, options),
            Self::In(m) => m.is_match(subfields, options),
            Self::Regex(m) => m.is_match(subfields, options),
            Self::Relation(m) => m.is_match(subfields, options),
        }
    }
}

/// A matcher that allows grouping, negation and connecting of singleton
/// matcher.
#[derive(Debug, PartialEq, Eq)]
pub enum SubfieldMatcher {
    Singleton(SingletonMatcher),
    Group(Box<SubfieldMatcher>),
    Not(Box<SubfieldMatcher>),
    Composite {
        lhs: Box<SubfieldMatcher>,
        op: BooleanOp,
        rhs: Box<SubfieldMatcher>,
    },
}

impl BitAnd for SubfieldMatcher {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::And,
            rhs: Box::new(rhs),
        }
    }
}

impl BitOr for SubfieldMatcher {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Or,
            rhs: Box::new(rhs),
        }
    }
}

#[inline]
fn parse_subfield_exists_matcher(
    i: &[u8],
) -> ParseResult<SubfieldMatcher> {
    map(parse_exists_matcher, |matcher| {
        SubfieldMatcher::Singleton(SingletonMatcher::Exists(matcher))
    })(i)
}

#[inline]
pub(crate) fn parse_subfield_singleton_matcher(
    i: &[u8],
) -> ParseResult<SubfieldMatcher> {
    map(parse_singleton_matcher, SubfieldMatcher::Singleton)(i)
}

fn parse_not_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        preceded(
            ws(char('!')),
            cut(alt((
                parse_group_matcher,
                parse_subfield_exists_matcher,
                parse_not_matcher,
            ))),
        ),
        |matcher| SubfieldMatcher::Not(Box::new(matcher)),
    )(i)
}

fn parse_group_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(
                alt((
                    parse_composite_matcher,
                    parse_subfield_singleton_matcher,
                    parse_not_matcher,
                    parse_group_matcher,
                )),
                ws(char(')')),
            )),
        ),
        |matcher| SubfieldMatcher::Group(Box::new(matcher)),
    )(i)
}

fn parse_or_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_group_matcher),
            ws(parse_and_matcher),
            ws(parse_subfield_singleton_matcher),
            ws(parse_not_matcher),
        )),
        many1(preceded(
            ws(tag("||")),
            cut(alt((
                ws(parse_group_matcher),
                ws(parse_and_matcher),
                ws(parse_subfield_singleton_matcher),
                ws(parse_not_matcher),
            ))),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev | next),
    ))
}

fn parse_and_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_group_matcher),
            map(
                ws(parse_singleton_matcher),
                SubfieldMatcher::Singleton,
            ),
            ws(parse_not_matcher),
        )),
        many1(preceded(
            ws(tag("&&")),
            alt((
                ws(parse_group_matcher),
                map(
                    ws(parse_singleton_matcher),
                    SubfieldMatcher::Singleton,
                ),
                ws(parse_not_matcher),
            )),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev & next),
    ))
}

#[inline]
fn parse_composite_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    alt((parse_or_matcher, parse_and_matcher))(i)
}

pub fn parse_subfield_matcher(
    i: &[u8],
) -> ParseResult<SubfieldMatcher> {
    alt((
        parse_composite_matcher,
        parse_group_matcher,
        parse_not_matcher,
        map(parse_singleton_matcher, SubfieldMatcher::Singleton),
    ))(i)
}

impl SubfieldMatcher {
    /// Create a new matcher from a string slice.
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_subfield_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }
}

impl Matcher for SubfieldMatcher {
    fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Singleton(m) => m.is_match(subfields, options),
            Self::Group(m) => m.is_match(subfields, options),
            Self::Not(m) => !m.is_match(subfields, options),
            Self::Composite { lhs, op, rhs } => {
                if *op == BooleanOp::And {
                    lhs.is_match(subfields.clone(), options)
                        && rhs.is_match(subfields, options)
                } else {
                    lhs.is_match(subfields.clone(), options)
                        || rhs.is_match(subfields, options)
                }
            }
        }
    }
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
            parse_subfield_codes(b"*"),
            SUBFIELD_CODES.chars().collect::<Vec<char>>()
        );

        assert_error!(parse_subfield_codes(b"!"));
        assert_error!(parse_subfield_codes(b"[a1!]"));
    }

    #[test]
    fn test_parse_relation_matcher() {
        assert_finished_and_eq!(
            parse_relation_matcher(b"0 == 'abc'"),
            RelationMatcher {
                codes: vec!['0'],
                op: RelationalOp::Eq,
                value: "abc".into()
            }
        );
        assert_finished_and_eq!(
            parse_relation_matcher(b"[012] =^ 'abc'"),
            RelationMatcher {
                codes: vec!['0', '1', '2'],
                op: RelationalOp::StartsWith,
                value: "abc".into()
            }
        );
        assert_finished_and_eq!(
            parse_relation_matcher(b"0 !^ 'T'"),
            RelationMatcher {
                codes: vec!['0'],
                op: RelationalOp::StartsNotWith,
                value: "T".into()
            }
        );
        assert_finished_and_eq!(
            parse_relation_matcher(b"0 =$ 'abc'"),
            RelationMatcher {
                codes: vec!['0'],
                op: RelationalOp::EndsWith,
                value: "abc".into()
            }
        );
        assert_finished_and_eq!(
            parse_relation_matcher(b"0 !$ 'z'"),
            RelationMatcher {
                codes: vec!['0'],
                op: RelationalOp::EndsNotWith,
                value: "z".into()
            }
        );
        assert_finished_and_eq!(
            parse_relation_matcher(b"0 =* 'abc'"),
            RelationMatcher {
                codes: vec!['0'],
                op: RelationalOp::Similar,
                value: "abc".into()
            }
        );

        assert_error!(parse_relation_matcher(b"0 >= 'abc'"));
        assert_error!(parse_relation_matcher(b"0 > 'abc'"));
        assert_error!(parse_relation_matcher(b"0 <= 'abc'"));
        assert_error!(parse_relation_matcher(b"0 < 'abc'"));
    }

    #[test]
    fn test_parse_regex_matcher() {
        assert_finished_and_eq!(
            parse_regex_matcher(b"0 =~ '^a.*c$'"),
            RegexMatcher {
                codes: vec!['0'],
                pattern: "^a.*c$".into(),
                invert: false,
            }
        );
        assert_finished_and_eq!(
            parse_regex_matcher(b"0 !~ '^a.*c$'"),
            RegexMatcher {
                codes: vec!['0'],
                pattern: "^a.*c$".into(),
                invert: true,
            }
        );

        assert_error!(parse_regex_matcher(b"0 =~ '^[ab$'"));
        assert_error!(parse_regex_matcher(b"0 !~ '^[ab$'"));
    }

    #[test]
    fn test_parse_in_matcher() {
        assert_finished_and_eq!(
            parse_in_matcher(b"0 in ['abc', 'bcd']"),
            InMatcher {
                codes: vec!['0'],
                values: vec!["abc".into(), "bcd".into()],
                invert: false
            }
        );

        assert_finished_and_eq!(
            parse_in_matcher(b"[09] not in ['abc', 'bcd']"),
            InMatcher {
                codes: vec!['0', '9'],
                values: vec!["abc".into(), "bcd".into()],
                invert: true,
            }
        );

        assert_error!(parse_in_matcher(b"0 in []"));
    }

    #[test]
    fn test_parse_cardinality_matcher() {
        assert_finished_and_eq!(
            parse_cardinality_matcher(b"#0 == 1"),
            CardinalityMatcher {
                code: '0',
                op: RelationalOp::Eq,
                value: 1,
            }
        );
        assert_finished_and_eq!(
            parse_cardinality_matcher(b"#0 >= 1"),
            CardinalityMatcher {
                code: '0',
                op: RelationalOp::Ge,
                value: 1,
            }
        );
        assert_finished_and_eq!(
            parse_cardinality_matcher(b"#0 > 1"),
            CardinalityMatcher {
                code: '0',
                op: RelationalOp::Gt,
                value: 1,
            }
        );
        assert_finished_and_eq!(
            parse_cardinality_matcher(b"#0 <= 1"),
            CardinalityMatcher {
                code: '0',
                op: RelationalOp::Le,
                value: 1,
            }
        );
        assert_finished_and_eq!(
            parse_cardinality_matcher(b"#0 < 1"),
            CardinalityMatcher {
                code: '0',
                op: RelationalOp::Lt,
                value: 1,
            }
        );

        assert_error!(parse_cardinality_matcher(b"#a =~ '^abc'"));
        assert_error!(parse_cardinality_matcher(b"#[ab] > 0"));
        assert_error!(parse_cardinality_matcher(b"#a > -1"));
    }
}
