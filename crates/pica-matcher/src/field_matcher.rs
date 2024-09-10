//! Matcher that works on PICA+ [Fields](pica_record_v1::Field).

use std::cell::RefCell;
use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::str::FromStr;

use bstr::ByteSlice;
use pica_record_v1::FieldRef;
use winnow::ascii::digit1;
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, terminated,
};
use winnow::error::ParserError;
use winnow::prelude::*;

use crate::common::{
    parse_quantifier, parse_relational_op_usize, ws, BooleanOp,
    RelationalOp,
};
use crate::occurrence_matcher::parse_occurrence_matcher;
use crate::subfield_matcher::{
    self, parse_subfield_matcher, parse_subfield_singleton_matcher,
};
use crate::tag_matcher::parse_tag_matcher;
use crate::{
    MatcherOptions, OccurrenceMatcher, ParseMatcherError, Quantifier,
    SubfieldMatcher, TagMatcher,
};

/// A field matcher that checks if a field exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistsMatcher {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
}

impl ExistsMatcher {
    /// Create a new exists matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::ExistsMatcher;
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = ExistsMatcher::new("003@?");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
    ///         &options
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &FieldRef::new("002@", None, vec![('0', "123456789X")]),
    ///         &options
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &T) -> Self {
        Self::try_from(data.as_ref()).expect("exists matcher")
    }

    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>> + Clone,
        _options: &MatcherOptions,
    ) -> bool {
        fields.into_iter().any(|field| {
            self.tag_matcher == field.tag()
                && self.occurrence_matcher == field.occurrence()
        })
    }
}

/// Parse a exists matcher expression.
fn parse_exists_matcher(i: &mut &[u8]) -> PResult<ExistsMatcher> {
    terminated(ws((parse_tag_matcher, parse_occurrence_matcher)), '?')
        .map(|(t, o)| ExistsMatcher {
            tag_matcher: t,
            occurrence_matcher: o,
        })
        .parse_next(i)
}

impl TryFrom<&[u8]> for ExistsMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_exists_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

impl FromStr for ExistsMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A field matcher that checks for fields satisfies subfield
/// criterion.
#[derive(Debug, Clone)]
pub struct SubfieldsMatcher {
    quantifier: Quantifier,
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: SubfieldMatcher,
}

impl SubfieldsMatcher {
    /// Create a new subfields matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::SubfieldsMatcher;
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SubfieldsMatcher::new("002@.0 == 'Olfo'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("002@", None, vec![('0', "Olfo")]),
    ///         &options
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &FieldRef::new("002@", None, vec![('0', "Oaf")]),
    ///         &options
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &T) -> Self {
        Self::try_from(data.as_ref()).expect("subfields matcher")
    }

    /// Returns `true` if at least one field exists with a matching tag
    /// and occurrence and a subfield matching the subfield
    /// matcher's criteria.
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let mut fields = fields.into_iter().filter(|field| {
            self.tag_matcher == field.tag()
                && self.occurrence_matcher == field.occurrence()
        });

        let check_fn = |field: &FieldRef| -> bool {
            self.subfield_matcher.is_match(field.subfields(), options)
        };

        match self.quantifier {
            Quantifier::All => fields.all(check_fn),
            Quantifier::Any => fields.any(check_fn),
        }
    }
}

fn parse_subfields_matcher_dot(
    i: &mut &[u8],
) -> PResult<SubfieldsMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        parse_tag_matcher,
        parse_occurrence_matcher,
        preceded('.', parse_subfield_singleton_matcher),
    )
        .map(|(q, t, o, s)| SubfieldsMatcher {
            quantifier: q,
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: s,
        })
        .parse_next(i)
}

fn parse_subfields_matcher_bracket(
    i: &mut &[u8],
) -> PResult<SubfieldsMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        parse_tag_matcher,
        parse_occurrence_matcher,
        delimited(ws('{'), parse_subfield_matcher, ws('}')),
    )
        .map(|(q, t, o, s)| SubfieldsMatcher {
            quantifier: q,
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: s,
        })
        .parse_next(i)
}

fn parse_subfields_matcher(i: &mut &[u8]) -> PResult<SubfieldsMatcher> {
    alt((parse_subfields_matcher_dot, parse_subfields_matcher_bracket))
        .parse_next(i)
}

impl TryFrom<&[u8]> for SubfieldsMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_subfields_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

impl FromStr for SubfieldsMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A field matcher that checks for the singleton matcher.
#[derive(Debug, Clone)]
pub enum SingletonMatcher {
    Exists(ExistsMatcher),
    Subfields(SubfieldsMatcher),
}

impl SingletonMatcher {
    /// Create a new singleton matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::SingletonMatcher;
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SingletonMatcher::new("003@?");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
    ///         &options
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &T) -> Self {
        Self::try_from(data.as_ref()).expect("singleton macher")
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Subfields(m) => m.is_match(fields, options),
            Self::Exists(m) => m.is_match(fields, options),
        }
    }
}

/// Parse a singleton matcher expression.
fn parse_singleton_matcher(i: &mut &[u8]) -> PResult<SingletonMatcher> {
    alt((
        parse_exists_matcher.map(SingletonMatcher::Exists),
        parse_subfields_matcher.map(SingletonMatcher::Subfields),
    ))
    .parse_next(i)
}

impl TryFrom<&[u8]> for SingletonMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_singleton_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

impl FromStr for SingletonMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A field matcher that checks the number of occurrences of a field.
#[derive(Debug, Clone)]
pub struct CardinalityMatcher {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    op: RelationalOp,
    value: usize,
}

impl CardinalityMatcher {
    /// Create a new cardinality matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::CardinalityMatcher;
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher =
    ///         CardinalityMatcher::new("#003@{0 == '123456789X'} >= 1");
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &T) -> Self {
        Self::try_from(data.as_ref()).expect("cardinality matcher")
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let count = fields
            .into_iter()
            .filter(|field| {
                self.tag_matcher == field.tag()
                    && self.occurrence_matcher == field.occurrence()
            })
            .filter(|field| {
                if let Some(ref matcher) = self.subfield_matcher {
                    matcher.is_match(field.subfields(), options)
                } else {
                    true
                }
            })
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

/// Parse a cardinality matcher expressions.
fn parse_cardinality_matcher(
    i: &mut &[u8],
) -> PResult<CardinalityMatcher> {
    preceded(
        ws('#'),
        (
            ws(parse_tag_matcher),
            ws(parse_occurrence_matcher),
            opt(delimited('{', parse_subfield_matcher, ws('}'))),
            ws(parse_relational_op_usize),
            digit1
                .verify_map(|value| std::str::from_utf8(value).ok())
                .verify_map(|value| value.parse::<usize>().ok()),
        ),
    )
    .map(|(t, o, s, op, value)| CardinalityMatcher {
        tag_matcher: t,
        occurrence_matcher: o,
        subfield_matcher: s,
        op,
        value,
    })
    .parse_next(i)
}

impl TryFrom<&[u8]> for CardinalityMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_cardinality_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

impl FromStr for CardinalityMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A field matcher that allows grouping, negation and connecting of
/// singleton matcher.
#[derive(Debug, Clone)]
pub enum FieldMatcher {
    Singleton(SingletonMatcher),
    Cardinality(CardinalityMatcher),
    Group(Box<FieldMatcher>),
    Not(Box<FieldMatcher>),
    Composite {
        lhs: Box<FieldMatcher>,
        op: BooleanOp,
        rhs: Box<FieldMatcher>,
    },
}

impl FieldMatcher {
    /// Create a new field matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::FieldMatcher;
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = FieldMatcher::new("003@?");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
    ///         &options
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &T) -> Self {
        Self::try_from(data.as_ref()).expect("field matcher")
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Singleton(m) => m.is_match(fields, options),
            Self::Group(m) => m.is_match(fields, options),
            Self::Not(m) => !m.is_match(fields, options),
            Self::Cardinality(m) => m.is_match(fields, options),
            Self::Composite { lhs, op, rhs } => match op {
                BooleanOp::And => {
                    lhs.is_match(fields.clone(), options)
                        && rhs.is_match(fields, options)
                }
                BooleanOp::Or => {
                    lhs.is_match(fields.clone(), options)
                        || rhs.is_match(fields, options)
                }
                BooleanOp::Xor => {
                    lhs.is_match(fields.clone(), options)
                        != rhs.is_match(fields, options)
                }
            },
        }
    }
}

/// Parse a singleton matcher expression (curly bracket notation).
#[inline]
fn parse_singleton_matcher_bracket(
    i: &mut &[u8],
) -> PResult<SingletonMatcher> {
    parse_subfields_matcher_bracket
        .map(SingletonMatcher::Subfields)
        .parse_next(i)
}

/// Parse field matcher singleton expression.
#[inline]
fn parse_field_matcher_singleton(
    i: &mut &[u8],
) -> PResult<FieldMatcher> {
    parse_singleton_matcher
        .map(FieldMatcher::Singleton)
        .parse_next(i)
}

/// Parse field matcher expression (curly bracket notation).
#[inline]
fn parse_field_matcher_singleton_bracket(
    i: &mut &[u8],
) -> PResult<FieldMatcher> {
    parse_singleton_matcher_bracket
        .map(FieldMatcher::Singleton)
        .parse_next(i)
}

/// Parse field matcher exists expression.
#[inline]
fn parse_field_matcher_exists(i: &mut &[u8]) -> PResult<FieldMatcher> {
    alt((
        parse_exists_matcher.map(|matcher| {
            FieldMatcher::Singleton(SingletonMatcher::Exists(matcher))
        }),
        (
            opt(parse_quantifier).map(Option::unwrap_or_default),
            parse_tag_matcher,
            parse_occurrence_matcher,
            preceded(ws('.'), subfield_matcher::parse_exists_matcher),
        )
            .map(|(q, t, o, s)| {
                FieldMatcher::Singleton(SingletonMatcher::Subfields(
                    SubfieldsMatcher {
                        quantifier: q,
                        tag_matcher: t,
                        occurrence_matcher: o,
                        subfield_matcher: SubfieldMatcher::Singleton(
                            subfield_matcher::SingletonMatcher::Exists(
                                s,
                            ),
                        ),
                    },
                ))
            }),
    ))
    .parse_next(i)
}

/// Parse field matcher cardinality expression.
#[inline]
fn parse_field_matcher_cardinality(
    i: &mut &[u8],
) -> PResult<FieldMatcher> {
    parse_cardinality_matcher
        .map(FieldMatcher::Cardinality)
        .parse_next(i)
}

thread_local! {
    pub static GROUP_LEVEL: RefCell<u32> = const { RefCell::new(0) };
}

fn increment_group_level(i: &mut &[u8]) -> PResult<()> {
    GROUP_LEVEL.with(|level| {
        *level.borrow_mut() += 1;
        if *level.borrow() >= 32 {
            Err(winnow::error::ErrMode::from_error_kind(
                i,
                winnow::error::ErrorKind::Many,
            ))
        } else {
            Ok(())
        }
    })
}

fn decrement_group_level() {
    GROUP_LEVEL.with(|level| {
        *level.borrow_mut() -= 1;
    })
}

#[inline]
fn parse_field_matcher_group(i: &mut &[u8]) -> PResult<FieldMatcher> {
    delimited(
        terminated(ws('('), increment_group_level),
        alt((
            parse_field_matcher_composite,
            parse_field_matcher_singleton,
            parse_field_matcher_not,
            parse_field_matcher_cardinality,
            parse_field_matcher_group,
        )),
        ws(')').map(|_| decrement_group_level()),
    )
    .map(|matcher| FieldMatcher::Group(Box::new(matcher)))
    .parse_next(i)
}

#[inline]
fn parse_field_matcher_not(i: &mut &[u8]) -> PResult<FieldMatcher> {
    preceded(
        ws('!'),
        alt((
            parse_field_matcher_group,
            parse_field_matcher_singleton_bracket,
            parse_field_matcher_exists,
            parse_field_matcher_not,
        )),
    )
    .map(|matcher| FieldMatcher::Not(Box::new(matcher)))
    .parse_next(i)
}

#[inline]
fn parse_field_matcher_xor(i: &mut &[u8]) -> PResult<FieldMatcher> {
    (
        ws(alt((
            parse_field_matcher_group,
            parse_field_matcher_and,
            parse_field_matcher_cardinality,
            parse_field_matcher_singleton,
            parse_field_matcher_not,
            parse_field_matcher_exists,
        ))),
        repeat(
            1..,
            preceded(
                ws(alt(("^", "XOR"))),
                ws(alt((
                    parse_field_matcher_group,
                    parse_field_matcher_and,
                    parse_field_matcher_cardinality,
                    parse_field_matcher_singleton,
                    parse_field_matcher_not,
                    parse_field_matcher_exists,
                ))),
            ),
        ),
    )
        .map(|(head, remainder): (_, Vec<_>)| {
            remainder.into_iter().fold(head, |prev, next| prev ^ next)
        })
        .parse_next(i)
}

#[inline]
fn parse_field_matcher_and(i: &mut &[u8]) -> PResult<FieldMatcher> {
    (
        ws(alt((
            parse_field_matcher_group,
            parse_field_matcher_cardinality,
            parse_field_matcher_singleton,
            parse_field_matcher_not,
            parse_field_matcher_exists,
        ))),
        repeat(
            1..,
            preceded(
                ws("&&"),
                ws(alt((
                    parse_field_matcher_group,
                    parse_field_matcher_cardinality,
                    parse_field_matcher_singleton,
                    parse_field_matcher_not,
                    parse_field_matcher_exists,
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
fn parse_field_matcher_or(i: &mut &[u8]) -> PResult<FieldMatcher> {
    (
        ws(alt((
            parse_field_matcher_group,
            parse_field_matcher_and,
            parse_field_matcher_xor,
            parse_field_matcher_cardinality,
            parse_field_matcher_singleton,
            parse_field_matcher_not,
            parse_field_matcher_exists,
        ))),
        repeat(
            1..,
            preceded(
                ws("||"),
                ws(alt((
                    parse_field_matcher_group,
                    parse_field_matcher_and,
                    parse_field_matcher_xor,
                    parse_field_matcher_cardinality,
                    parse_field_matcher_singleton,
                    parse_field_matcher_not,
                    parse_field_matcher_exists,
                ))),
            ),
        ),
    )
        .map(|(head, remainder): (_, Vec<_>)| {
            remainder.into_iter().fold(head, |prev, next| prev | next)
        })
        .parse_next(i)
}

fn parse_field_matcher_composite(
    i: &mut &[u8],
) -> PResult<FieldMatcher> {
    alt((
        parse_field_matcher_or,
        parse_field_matcher_xor,
        parse_field_matcher_and,
    ))
    .parse_next(i)
}

pub fn parse_field_matcher(i: &mut &[u8]) -> PResult<FieldMatcher> {
    ws(alt((
        parse_field_matcher_composite,
        parse_field_matcher_group,
        parse_field_matcher_not,
        parse_field_matcher_singleton,
        parse_field_matcher_cardinality,
    )))
    .parse_next(i)
}

impl TryFrom<&[u8]> for FieldMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_field_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

impl FromStr for FieldMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

impl BitAnd for FieldMatcher {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::And,
            rhs: Box::new(rhs),
        }
    }
}

impl BitOr for FieldMatcher {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Or,
            rhs: Box::new(rhs),
        }
    }
}

impl BitXor for FieldMatcher {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Xor,
            rhs: Box::new(rhs),
        }
    }
}

impl Not for FieldMatcher {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}
