//! Matcher that works on PICA+ [Fields](pica_record::Field).

use std::ops::{BitAnd, BitOr, Not};

use bstr::ByteSlice;
use pica_record::Field;
use winnow::ascii::digit1;
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, terminated,
};
use winnow::prelude::*;

use crate::common::{
    parse_relational_op_usize, ws, BooleanOp, RelationalOp,
};
use crate::occurrence_matcher::parse_occurrence_matcher;
use crate::subfield_matcher::{
    self, parse_subfield_matcher, parse_subfield_singleton_matcher,
};
use crate::tag_matcher::parse_tag_matcher;
use crate::{
    MatcherOptions, OccurrenceMatcher, ParseMatcherError,
    SubfieldMatcher, TagMatcher,
};

/// A field matcher that checks if a field exists.
#[derive(Debug, PartialEq, Eq)]
pub struct ExistsMatcher<'a> {
    tag_matcher: TagMatcher<'a>,
    occurrence_matcher: OccurrenceMatcher<'a>,
}

impl<'a> ExistsMatcher<'a> {
    /// Create a new exists matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::ExistsMatcher;
    /// use pica_record::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = ExistsMatcher::new("003@?");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         &Field::new("003@", None, vec![('0', "123456789X")]),
    ///         &options
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &Field::new("002@", None, vec![('0', "123456789X")]),
    ///         &options
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &'a T) -> Self {
        Self::try_from(data.as_ref()).expect("exists matcher")
    }

    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    pub fn is_match<'b>(
        &self,
        fields: impl IntoIterator<Item = &'b Field<'b>> + Clone,
        _options: &MatcherOptions,
    ) -> bool {
        fields.into_iter().any(|field| {
            self.tag_matcher == field.tag()
                && self.occurrence_matcher == field.occurrence()
        })
    }
}

/// Parse a exists matcher expression.
fn parse_exists_matcher<'a>(
    i: &mut &'a [u8],
) -> PResult<ExistsMatcher<'a>> {
    terminated(ws((parse_tag_matcher, parse_occurrence_matcher)), '?')
        .map(|(t, o)| ExistsMatcher {
            tag_matcher: t,
            occurrence_matcher: o,
        })
        .parse_next(i)
}

impl<'a> TryFrom<&'a [u8]> for ExistsMatcher<'a> {
    type Error = ParseMatcherError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        parse_exists_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

/// A field matcher that checks for fields satisfies subfield
/// criterion.
#[derive(Debug)]
pub struct SubfieldsMatcher<'a> {
    tag_matcher: TagMatcher<'a>,
    occurrence_matcher: OccurrenceMatcher<'a>,
    subfield_matcher: SubfieldMatcher,
}

impl<'a> SubfieldsMatcher<'a> {
    /// Create a new subfields matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::SubfieldsMatcher;
    /// use pica_record::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SubfieldsMatcher::new("002@.0 == 'Olfo'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         &Field::new("002@", None, vec![('0', "Olfo")]),
    ///         &options
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &Field::new("002@", None, vec![('0', "Oaf")]),
    ///         &options
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &'a T) -> Self {
        Self::try_from(data.as_ref()).expect("subfields matcher")
    }

    /// Returns `true` if at least one field exists with a matching tag
    /// and occurrence and a subfield matching the subfield
    /// matcher's criteria.
    pub fn is_match<'b>(
        &self,
        fields: impl IntoIterator<Item = &'b Field<'b>>,
        options: &MatcherOptions,
    ) -> bool {
        fields.into_iter().any(|field| {
            self.tag_matcher == field.tag()
                && self.occurrence_matcher == field.occurrence()
                && self
                    .subfield_matcher
                    .is_match(field.subfields(), options)
        })
    }
}

fn parse_subfields_matcher_dot<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldsMatcher<'a>> {
    (
        parse_tag_matcher,
        parse_occurrence_matcher,
        preceded(
            alt((
                '.',
                ws('$'), // FIXME: remove legacy snytax
            )),
            parse_subfield_singleton_matcher,
        ),
    )
        .map(|(t, o, s)| SubfieldsMatcher {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: s,
        })
        .parse_next(i)
}

fn parse_subfields_matcher_bracket<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldsMatcher<'a>> {
    (
        parse_tag_matcher,
        parse_occurrence_matcher,
        delimited(ws('{'), parse_subfield_matcher, ws('}')),
    )
        .map(|(t, o, s)| SubfieldsMatcher {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: s,
        })
        .parse_next(i)
}

fn parse_subfields_matcher<'a>(
    i: &mut &'a [u8],
) -> PResult<SubfieldsMatcher<'a>> {
    alt((parse_subfields_matcher_dot, parse_subfields_matcher_bracket))
        .parse_next(i)
}

impl<'a> TryFrom<&'a [u8]> for SubfieldsMatcher<'a> {
    type Error = ParseMatcherError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        parse_subfields_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

/// A field matcher that checks for the singleton matcher.
#[derive(Debug)]
pub enum SingletonMatcher<'a> {
    Exists(ExistsMatcher<'a>),
    Subfields(SubfieldsMatcher<'a>),
}

impl<'a> SingletonMatcher<'a> {
    /// Create a new singleton matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::SingletonMatcher;
    /// use pica_record::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SingletonMatcher::new("003@?");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         &Field::new("003@", None, vec![('0', "123456789X")]),
    ///         &options
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &'a T) -> Self {
        Self::try_from(data.as_ref()).expect("singleton macher")
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'b>(
        &self,
        fields: impl IntoIterator<Item = &'b Field<'b>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Subfields(m) => m.is_match(fields, options),
            Self::Exists(m) => m.is_match(fields, options),
        }
    }
}

/// Parse a singleton matcher expression.
fn parse_singleton_matcher<'a>(
    i: &mut &'a [u8],
) -> PResult<SingletonMatcher<'a>> {
    alt((
        parse_exists_matcher.map(SingletonMatcher::Exists),
        parse_subfields_matcher.map(SingletonMatcher::Subfields),
    ))
    .parse_next(i)
}

impl<'a> TryFrom<&'a [u8]> for SingletonMatcher<'a> {
    type Error = ParseMatcherError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        parse_singleton_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

/// A field matcher that checks the number of occurrences of a field.
#[derive(Debug)]
pub struct CardinalityMatcher<'a> {
    tag_matcher: TagMatcher<'a>,
    occurrence_matcher: OccurrenceMatcher<'a>,
    subfield_matcher: Option<SubfieldMatcher>,
    op: RelationalOp,
    value: usize,
}

impl<'a> CardinalityMatcher<'a> {
    /// Create a new cardinality matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::CardinalityMatcher;
    /// use pica_record::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher =
    ///         CardinalityMatcher::new("#003@{0 == '123456789X'} >= 1");
    ///
    ///     assert!(matcher.is_match(
    ///         &Field::new("003@", None, vec![('0', "123456789X")]),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &'a T) -> Self {
        Self::try_from(data.as_ref()).expect("cardinality matcher")
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'b>(
        &self,
        fields: impl IntoIterator<Item = &'b Field<'b>>,
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
fn parse_cardinality_matcher<'a>(
    i: &mut &'a [u8],
) -> PResult<CardinalityMatcher<'a>> {
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

impl<'a> TryFrom<&'a [u8]> for CardinalityMatcher<'a> {
    type Error = ParseMatcherError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        parse_cardinality_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

/// A field matcher that allows grouping, negation and connecting of
/// singleton matcher.
#[derive(Debug)]
pub enum FieldMatcher<'a> {
    Singleton(SingletonMatcher<'a>),
    Cardinality(CardinalityMatcher<'a>),
    Group(Box<FieldMatcher<'a>>),
    Not(Box<FieldMatcher<'a>>),
    Composite {
        lhs: Box<FieldMatcher<'a>>,
        op: BooleanOp,
        rhs: Box<FieldMatcher<'a>>,
    },
}

impl<'a> FieldMatcher<'a> {
    /// Create a new field matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::FieldMatcher;
    /// use pica_record::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = FieldMatcher::new("003@?");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         &Field::new("003@", None, vec![('0', "123456789X")]),
    ///         &options
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &'a T) -> Self {
        Self::try_from(data.as_ref()).expect("field matcher")
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'b>(
        &self,
        fields: impl IntoIterator<Item = &'b Field<'b>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Singleton(m) => m.is_match(fields, options),
            Self::Group(m) => m.is_match(fields, options),
            Self::Not(m) => !m.is_match(fields, options),
            Self::Cardinality(m) => m.is_match(fields, options),
            Self::Composite { lhs, op, rhs } => {
                if *op == BooleanOp::And {
                    lhs.is_match(fields.clone(), options)
                        && rhs.is_match(fields, options)
                } else {
                    lhs.is_match(fields.clone(), options)
                        || rhs.is_match(fields, options)
                }
            }
        }
    }
}

/// Parse a singleton matcher expression (curly bracket notation).
#[inline]
fn parse_singleton_matcher_bracket<'a>(
    i: &mut &'a [u8],
) -> PResult<SingletonMatcher<'a>> {
    parse_subfields_matcher_bracket
        .map(SingletonMatcher::Subfields)
        .parse_next(i)
}

/// Parse field matcher singleton expression.
#[inline]
fn parse_field_matcher_singleton<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
    parse_singleton_matcher
        .map(FieldMatcher::Singleton)
        .parse_next(i)
}

/// Parse field matcher expression (curly bracket notation).
#[inline]
fn parse_field_matcher_singleton_bracket<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
    parse_singleton_matcher_bracket
        .map(FieldMatcher::Singleton)
        .parse_next(i)
}

/// Parse field matcher exists expression.
#[inline]
fn parse_field_matcher_exists<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
    alt((
        parse_exists_matcher.map(|matcher| {
            FieldMatcher::Singleton(SingletonMatcher::Exists(matcher))
        }),
        (
            parse_tag_matcher,
            parse_occurrence_matcher,
            preceded(ws('.'), subfield_matcher::parse_exists_matcher),
        )
            .map(|(t, o, s)| {
                FieldMatcher::Singleton(SingletonMatcher::Subfields(
                    SubfieldsMatcher {
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
fn parse_field_matcher_cardinality<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
    parse_cardinality_matcher
        .map(FieldMatcher::Cardinality)
        .parse_next(i)
}

#[inline]
fn parse_field_matcher_group<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
    delimited(
        ws('('),
        alt((
            parse_field_matcher_composite,
            parse_field_matcher_singleton,
            parse_field_matcher_not,
            parse_field_matcher_cardinality,
            parse_field_matcher_group,
        )),
        ws(')'),
    )
    .map(|matcher| FieldMatcher::Group(Box::new(matcher)))
    .parse_next(i)
}

#[inline]
fn parse_field_matcher_not<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
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
fn parse_field_matcher_and<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
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
fn parse_field_matcher_or<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
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
                ws("||"),
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
            remainder.into_iter().fold(head, |prev, next| prev | next)
        })
        .parse_next(i)
}

fn parse_field_matcher_composite<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
    alt((parse_field_matcher_or, parse_field_matcher_and)).parse_next(i)
}

pub fn parse_field_matcher<'a>(
    i: &mut &'a [u8],
) -> PResult<FieldMatcher<'a>> {
    ws(alt((
        parse_field_matcher_composite,
        parse_field_matcher_group,
        parse_field_matcher_not,
        parse_field_matcher_singleton,
        parse_field_matcher_cardinality,
    )))
    .parse_next(i)
}

impl<'a> TryFrom<&'a [u8]> for FieldMatcher<'a> {
    type Error = ParseMatcherError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        parse_field_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidFieldMatcher(value)
        })
    }
}

impl<'a> BitAnd for FieldMatcher<'a> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::And,
            rhs: Box::new(rhs),
        }
    }
}

impl<'a> BitOr for FieldMatcher<'a> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Or,
            rhs: Box::new(rhs),
        }
    }
}

impl<'a> Not for FieldMatcher<'a> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}
