//! Matcher that works on PICA+ [Subfields](pica_record::Subfield).

use std::ops::{BitAnd, BitOr};

use bstr::ByteSlice;
use pica_record::parser::parse_subfield_code;
use pica_record::Subfield;
use regex::bytes::{Regex, RegexBuilder};
use strsim::normalized_levenshtein;
use winnow::ascii::digit1;
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, separated, terminated,
};
use winnow::{PResult, Parser};

use crate::common::{
    parse_relational_op_str, parse_relational_op_usize, parse_string,
    ws, BooleanOp, RelationalOp,
};
use crate::{MatcherOptions, ParseMatcherError};

/// A matcher that checks if a subfield exists.
///
/// This matcher can be used to determine if a single subfield or a
/// list of subfields contains at least one subfield with a code, that
/// is contained in the matcher's code list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistsMatcher {
    codes: Vec<char>,
}

const SUBFIELD_CODES: &str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Parse a list of subfield codes
fn parse_subfield_codes(i: &mut &[u8]) -> PResult<Vec<char>> {
    alt((
        delimited('[', repeat(1.., parse_subfield_code), ']'),
        parse_subfield_code.map(|code| vec![code]),
        '*'.value(SUBFIELD_CODES.chars().collect()),
    ))
    .parse_next(i)
}

/// Parse the matcher expression from a byte slice.
fn parse_exists_matcher(i: &mut &[u8]) -> PResult<ExistsMatcher> {
    terminated(parse_subfield_codes, '?')
        .map(|codes| ExistsMatcher { codes })
        .parse_next(i)
}

impl ExistsMatcher {
    /// Create a new exists matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::ExistsMatcher;
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = ExistsMatcher::new("0?");
    ///
    ///     assert!(matcher.is_match(
    ///         &Subfield::new('0', "123456789X"),
    ///         &Default::default()
    ///     ));
    ///
    ///     assert!(!matcher
    ///         .is_match(&Subfield::new('a', "abc"), &Default::default()));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(value: T) -> Self {
        Self::try_from(value.as_ref()).expect("exists matcher")
    }

    /// Returns `true` if at least one subfield is found with a code
    /// which is in the matcher's code list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::ExistsMatcher;
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = ExistsMatcher::new("[103]?");
    ///     let options = Default::default();
    ///     assert!(matcher.is_match(&Subfield::new('0', "123"), &options));
    ///
    ///     let matcher = ExistsMatcher::new("*?");
    ///     let options = Default::default();
    ///     assert!(matcher.is_match(&Subfield::new('a', "abc"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<'a>>,
        _options: &MatcherOptions,
    ) -> bool {
        subfields
            .into_iter()
            .any(|subfield| self.codes.contains(&subfield.code()))
    }
}

impl TryFrom<&[u8]> for ExistsMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_exists_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

/// A matcher that checks relations between (string) values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationMatcher {
    codes: Vec<char>,
    op: RelationalOp,
    value: Vec<u8>,
}

/// Parse a relational expression
fn parse_relation_matcher(i: &mut &[u8]) -> PResult<RelationMatcher> {
    (
        ws(parse_subfield_codes),
        ws(parse_relational_op_str),
        ws(parse_string),
    )
        .map(|(codes, op, value)| RelationMatcher { codes, op, value })
        .parse_next(i)
}

impl RelationMatcher {
    /// Create a new relation matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::RelationMatcher;
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = RelationMatcher::new("0 == '123456789X'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher
    ///         .is_match(&Subfield::new('0', "123456789X"), &options));
    ///
    ///     assert!(!matcher
    ///         .is_match(&Subfield::new('0', "123456789!"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(value: T) -> Self {
        Self::try_from(value.as_ref()).expect("relation matcher")
    }

    /// Returns true if at least one subfield is found, when the
    /// subfield's value and the matcher value are related. The two
    /// values are related iff the relation defined by the operator
    /// exists.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        use RelationalOp::*;

        subfields
            .into_iter()
            .filter(|s| self.codes.contains(&s.code()))
            .any(|subfield| {
                let value = subfield.value().as_ref();
                match self.op {
                    Eq => self.compare(value, options),
                    Ne => !self.compare(value, options),
                    StartsWith => {
                        self.starts_with(value, options, false)
                    }
                    StartsNotWith => {
                        self.starts_with(value, options, true)
                    }
                    EndsWith => self.ends_with(value, options, false),
                    EndsNotWith => self.ends_with(value, options, true),
                    Similar => self.is_similar(value, options),
                    Contains => self.contains(value, options),
                    _ => unreachable!(),
                }
            })
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
    /// value, otherwise `false`. If the `case_ignore` flag is set,
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
    /// value, otherwise `false`. If the `case_ignore` flag is set,
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
    fn is_similar(&self, rhs: &[u8], options: &MatcherOptions) -> bool {
        let lhs = self.value.to_str_lossy();
        let rhs = rhs.to_str_lossy();

        let score = if options.case_ignore {
            normalized_levenshtein(
                &lhs.to_lowercase(),
                &rhs.to_lowercase(),
            )
        } else {
            normalized_levenshtein(&lhs, &rhs)
        };

        score > options.strsim_threshold
    }

    /// Returns `true` if the given value is a substring of the value.
    /// If the `case_ignore` flag is set, both strings will be
    /// converted to lowercase first.
    fn contains(&self, value: &[u8], options: &MatcherOptions) -> bool {
        if options.case_ignore {
            value
                .to_lowercase()
                .find(self.value.to_lowercase())
                .is_some()
        } else {
            value.find(&self.value).is_some()
        }
    }
}

impl TryFrom<&[u8]> for RelationMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_relation_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

/// A matcher that checks a subfield value against a regex.
#[derive(Clone, Debug)]
pub struct RegexMatcher {
    codes: Vec<char>,
    re: String,
    invert: bool,
}

impl RegexMatcher {
    /// Create a new regex matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::RegexMatcher;
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = RegexMatcher::new("0 =~ '^Oa'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(&Subfield::new('0', "Oa"), &options));
    ///
    ///     let matcher = RegexMatcher::new("0 !~ '^Oa'");
    ///     assert!(matcher.is_match(&Subfield::new('0', "Ob"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
        Self::try_from(data.as_ref()).expect("regex matcher")
    }

    /// Returns true if at least one subfield value is found, that
    /// matches against the regular expression.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let re = RegexBuilder::new(&self.re)
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

/// Parse a regex matcher expression
fn parse_regex_matcher(i: &mut &[u8]) -> PResult<RegexMatcher> {
    (
        ws(parse_subfield_codes),
        ws(alt(("=~".value(false), "!~".value(true)))),
        parse_string
            .verify_map(|re| String::from_utf8(re).ok())
            .verify(|re| Regex::new(re).is_ok()),
    )
        .map(|(codes, invert, re)| RegexMatcher { codes, invert, re })
        .parse_next(i)
}

impl TryFrom<&[u8]> for RegexMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_regex_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

/// A matcher that checks if a subfield value is in a predefined list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InMatcher {
    codes: Vec<char>,
    values: Vec<Vec<u8>>,
    invert: bool,
}

impl InMatcher {
    /// Create a new matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::InMatcher;
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = InMatcher::new("0 in ['abc', 'def']");
    ///     let options = Default::default();
    ///     assert!(matcher.is_match(&Subfield::new('0', "def"), &options));
    ///
    ///     let matcher = InMatcher::new("0 not in ['abc', 'def']");
    ///     let options = Default::default();
    ///     assert!(matcher.is_match(&Subfield::new('0', "hij"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
        Self::try_from(data.as_ref()).expect("in matcher")
    }

    /// Returns `true` if at least one subfield is found, where the
    /// value is contained in the matcher list.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        subfields
            .into_iter()
            .filter(|s| self.codes.contains(&s.code()))
            .any(|subfield| {
                let mut result = self.values.iter().any(|rhs| {
                    if options.case_ignore {
                        subfield.value().to_lowercase()
                            == rhs.to_lowercase()
                    } else {
                        subfield.value() == rhs
                    }
                });

                if self.invert {
                    result = !result;
                }

                result
            })
    }
}

/// Parse a in matcher expression.
fn parse_in_matcher(i: &mut &[u8]) -> PResult<InMatcher> {
    (
        ws(parse_subfield_codes),
        opt(ws("not")).map(|x| x.is_some()),
        preceded(
            ws("in"),
            delimited(
                ws('['),
                separated(1.., parse_string, ws(',')),
                ws(']'),
            ),
        ),
    )
        .map(|(codes, invert, values)| InMatcher {
            codes,
            invert,
            values,
        })
        .parse_next(i)
}

impl TryFrom<&[u8]> for InMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_in_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

/// A matcher that checks the number of occurrences of a subfield.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardinalityMatcher {
    code: char,
    op: RelationalOp,
    value: usize,
}

impl CardinalityMatcher {
    /// Create a new matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::CardinalityMatcher;
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = CardinalityMatcher::new("#0 > 1");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         vec![
    ///             &Subfield::new('0', "def"),
    ///             &Subfield::new('0', "abc")
    ///         ],
    ///         &options
    ///     ));
    ///
    ///     assert!(
    ///         !matcher.is_match(&Subfield::new('0', "def"), &options)
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
        Self::try_from(data.as_ref()).expect("cardinality matcher")
    }

    /// Returns true of number of fields with a code equal to the
    /// matcher's code is `==`, `!=`, `>=`, `>`, `<=`, or `<` than the
    /// matcher's value.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<'a>>,
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

/// Parse a cardinality matcher expression.
fn parse_cardinality_matcher(
    i: &mut &[u8],
) -> PResult<CardinalityMatcher> {
    preceded(
        ws('#'),
        (
            ws(parse_subfield_code),
            ws(parse_relational_op_usize),
            digit1
                .verify_map(|value| std::str::from_utf8(value).ok())
                .verify_map(|value| value.parse::<usize>().ok()),
        ),
    )
    .map(|(code, op, value)| CardinalityMatcher { code, op, value })
    .parse_next(i)
}

impl TryFrom<&[u8]> for CardinalityMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_cardinality_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

/// A matcher that checks for the singleton matcher.
///
/// This matcher combines all atomic, singleton matcher into a new
/// matcher.
#[derive(Clone, Debug)]
pub enum SingletonMatcher {
    Cardinality(CardinalityMatcher),
    Exists(ExistsMatcher),
    In(InMatcher),
    Regex(RegexMatcher),
    Relation(RelationMatcher),
}

/// Parse a singleton matcher expression.
fn parse_singleton_matcher(i: &mut &[u8]) -> PResult<SingletonMatcher> {
    alt((
        parse_cardinality_matcher.map(SingletonMatcher::Cardinality),
        parse_exists_matcher.map(SingletonMatcher::Exists),
        parse_in_matcher.map(SingletonMatcher::In),
        parse_regex_matcher.map(SingletonMatcher::Regex),
        parse_relation_matcher.map(SingletonMatcher::Relation),
    ))
    .parse_next(i)
}

impl SingletonMatcher {
    /// Create a new singleton matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::SingletonMatcher;
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SingletonMatcher::new("0 != '123456789X'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher
    ///         .is_match(&Subfield::new('0', "2345678901"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
        Self::try_from(data.as_ref()).expect("singleton matcher")
    }

    /// Returns `true` if the underlying matcher returns `true`.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<'a>>,
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

impl TryFrom<&[u8]> for SingletonMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_singleton_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

/// A matcher that allows grouping, negation and connecting of
/// singleton matcher.
#[derive(Clone, Debug)]
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

impl SubfieldMatcher {
    /// Create a new matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::SubfieldMatcher;
    /// use pica_record::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher =
    ///         SubfieldMatcher::new("0 != '123456789X' && 0 =^ '234'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher
    ///         .is_match(&Subfield::new('0', "2345678901"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
        Self::try_from(data.as_ref()).expect("subfield matcher")
    }

    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<'a>> + Clone,
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

#[inline]
fn parse_subfield_exists_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
    parse_exists_matcher
        .map(SingletonMatcher::Exists)
        .map(SubfieldMatcher::Singleton)
        .parse_next(i)
}

#[inline]
fn parse_subfield_singleton_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
    parse_singleton_matcher
        .map(SubfieldMatcher::Singleton)
        .parse_next(i)
}

#[inline]
fn parse_not_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    preceded(
        ws('!'),
        alt((
            parse_group_matcher,
            parse_subfield_exists_matcher,
            parse_not_matcher,
        )),
    )
    .map(|matcher| SubfieldMatcher::Not(Box::new(matcher)))
    .parse_next(i)
}

#[inline]
fn parse_group_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    delimited(
        ws('('),
        alt((
            parse_composite_matcher,
            parse_subfield_singleton_matcher,
            parse_not_matcher,
            parse_group_matcher,
        )),
        ws(')'),
    )
    .map(|matcher| SubfieldMatcher::Group(Box::new(matcher)))
    .parse_next(i)
}

#[inline]
fn parse_or_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    (
        alt((
            ws(parse_group_matcher),
            ws(parse_and_matcher),
            ws(parse_subfield_singleton_matcher),
            ws(parse_not_matcher),
        )),
        repeat(
            1..,
            preceded(
                ws("||"),
                alt((
                    ws(parse_group_matcher),
                    ws(parse_and_matcher),
                    ws(parse_subfield_singleton_matcher),
                    ws(parse_not_matcher),
                )),
            ),
        ),
    )
        .map(|(head, remainder): (_, Vec<_>)| {
            remainder.into_iter().fold(head, |prev, next| prev | next)
        })
        .parse_next(i)
}

#[inline]
fn parse_and_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    (
        ws(alt((
            parse_group_matcher,
            parse_singleton_matcher.map(SubfieldMatcher::Singleton),
            parse_not_matcher,
        ))),
        repeat(
            1..,
            preceded(
                ws("&&"),
                ws(alt((
                    parse_group_matcher,
                    parse_singleton_matcher
                        .map(SubfieldMatcher::Singleton),
                    parse_not_matcher,
                ))),
            ),
        ),
    )
        .map(|(head, remainder): (_, Vec<_>)| {
            remainder.into_iter().fold(head, |prev, next| prev & next)
        })
        .parse_next(i)
}

#[inline]
fn parse_composite_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    alt((parse_or_matcher, parse_and_matcher)).parse_next(i)
}

pub fn parse_subfield_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
    alt((
        parse_composite_matcher,
        parse_group_matcher,
        parse_not_matcher,
        parse_singleton_matcher.map(SubfieldMatcher::Singleton),
    ))
    .parse_next(i)
}

impl TryFrom<&[u8]> for SubfieldMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_subfield_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_subfield_codes() {
        use super::parse_subfield_codes;

        let codes = SUBFIELD_CODES.chars().collect::<Vec<char>>();

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_subfield_codes.parse($input).unwrap(),
                    $expected
                );
            };
        }

        for code in codes.iter() {
            parse_success!(code.to_string().as_bytes(), vec![*code]);
        }

        parse_success!(b"[12]", vec!['1', '2']);
        parse_success!(b"*", codes);

        assert!(parse_subfield_codes.parse(b"!").is_err());
        assert!(parse_subfield_codes.parse(b"12").is_err());
        assert!(parse_subfield_codes.parse(b"[a1!]").is_err());
    }
}
