//! Matcher that works on PICA+ [Fields](pica_record::Field).

use std::ops::{BitAnd, BitOr, Not};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{all_consuming, cut, map, map_res, opt};
use nom::multi::many1;
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::Finish;
use pica_record::parser::ParseResult;
use pica_record::Field;

use crate::common::{
    parse_relational_op_usize, ws, BooleanOp, RelationalOp,
};
use crate::occurrence_matcher::{
    parse_occurrence_matcher, OccurrenceMatcher,
};
use crate::subfield_matcher::{
    self, parse_subfield_matcher, parse_subfield_singleton_matcher,
    Matcher,
};
use crate::tag_matcher::parse_tag_matcher;
use crate::{
    MatcherOptions, ParseMatcherError, SubfieldMatcher, TagMatcher,
};

/// A field matcher that checks if a field exists.
#[derive(Debug, PartialEq, Eq)]
pub struct ExistsMatcher {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
}

/// Parse a exists matcher expression.
fn parse_exists_matcher(i: &[u8]) -> ParseResult<ExistsMatcher> {
    map(
        terminated(
            pair(ws(parse_tag_matcher), parse_occurrence_matcher),
            char('?'),
        ),
        |(t, o)| ExistsMatcher {
            tag_matcher: t,
            occurrence_matcher: o,
        },
    )(i)
}

impl ExistsMatcher {
    /// Create a new exists matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::ExistsMatcher;
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = ExistsMatcher::new("003@?")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
    ///         &Default::default()
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &FieldRef::new("002@", None, vec![('0', "123456789X")]),
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
                ParseMatcherError::InvalidFieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }

    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    pub fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        fields: impl IntoIterator<Item = &'a Field<T>> + Clone,
        _options: &MatcherOptions,
    ) -> bool {
        fields.into_iter().any(|field| {
            self.tag_matcher == field.tag()
                && self.occurrence_matcher == field.occurrence()
        })
    }
}

/// A field matcher that checks for fields statifies subfield criterion.
#[derive(Debug, PartialEq, Eq)]
pub struct SubfieldsMatcher {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: SubfieldMatcher,
}

/// Parse a subfields matcher expression.
fn parse_subfields_matcher_dot(
    i: &[u8],
) -> ParseResult<SubfieldsMatcher> {
    map(
        tuple((
            parse_tag_matcher,
            parse_occurrence_matcher,
            preceded(
                alt((char('.'), ws(char('$')))),
                parse_subfield_singleton_matcher,
            ),
        )),
        |(t, o, s)| SubfieldsMatcher {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: s,
        },
    )(i)
}

fn parse_subfields_matcher_bracket(
    i: &[u8],
) -> ParseResult<SubfieldsMatcher> {
    map(
        tuple((
            parse_tag_matcher,
            parse_occurrence_matcher,
            preceded(
                ws(char('{')),
                cut(terminated(parse_subfield_matcher, ws(char('}')))),
            ),
        )),
        |(t, o, s)| SubfieldsMatcher {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: s,
        },
    )(i)
}

fn parse_subfields_matcher(i: &[u8]) -> ParseResult<SubfieldsMatcher> {
    alt((parse_subfields_matcher_dot, parse_subfields_matcher_bracket))(
        i,
    )
}

impl SubfieldsMatcher {
    /// Create a new subfields matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::SubfieldsMatcher;
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SubfieldsMatcher::new("002@.0 == 'Olfo'")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("002@", None, vec![('0', "Olfo")]),
    ///         &Default::default()
    ///     ));
    ///
    ///     assert!(!matcher.is_match(
    ///         &FieldRef::new("002@", None, vec![('0', "Oaf")]),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_subfields_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidFieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }

    /// Returns `true` if at least one field exists with a matching tag
    /// and occurrence and a subfield matching the subfield matcher's
    /// criteria.
    pub fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        fields: impl IntoIterator<Item = &'a Field<T>> + Clone,
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

/// A field matcher that checks for the singleton matcher.
#[derive(Debug, PartialEq, Eq)]
pub enum SingletonMatcher {
    Exists(ExistsMatcher),
    Subfields(SubfieldsMatcher),
}

/// Parse a singleton matcher expression.
fn parse_singleton_matcher(i: &[u8]) -> ParseResult<SingletonMatcher> {
    alt((
        map(parse_exists_matcher, SingletonMatcher::Exists),
        map(parse_subfields_matcher, SingletonMatcher::Subfields),
    ))(i)
}

/// Parse a singleton matcher expression (curly bracket notation).
#[inline]
fn parse_singleton_matcher_bracket(
    i: &[u8],
) -> ParseResult<SingletonMatcher> {
    map(parse_subfields_matcher_bracket, SingletonMatcher::Subfields)(i)
}

impl SingletonMatcher {
    /// Create a new singleton matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::field_matcher::SingletonMatcher;
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SingletonMatcher::new("003@?")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
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
                ParseMatcherError::InvalidFieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        fields: impl IntoIterator<Item = &'a Field<T>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Exists(m) => m.is_match(fields, options),
            Self::Subfields(m) => m.is_match(fields, options),
        }
    }
}

/// A field matcher that checks the number of occurrences of a field.
#[derive(Debug, PartialEq, Eq)]
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
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher =
    ///         CardinalityMatcher::new("#003@{0 == '123456789X'} >= 1")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
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
                ParseMatcherError::InvalidFieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        fields: impl IntoIterator<Item = &'a Field<T>> + Clone,
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
    i: &[u8],
) -> ParseResult<CardinalityMatcher> {
    map(
        preceded(
            ws(char('#')),
            cut(tuple((
                ws(parse_tag_matcher),
                ws(parse_occurrence_matcher),
                opt(preceded(
                    ws(char('{')),
                    cut(terminated(
                        parse_subfield_matcher,
                        ws(char('}')),
                    )),
                )),
                ws(parse_relational_op_usize),
                map_res(digit1, |s| {
                    std::str::from_utf8(s).unwrap().parse::<usize>()
                }),
            ))),
        ),
        |(t, o, s, op, value)| CardinalityMatcher {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: s,
            op,
            value,
        },
    )(i)
}

/// A field matcher that allows grouping, negation and connecting of
/// singleton matcher.
#[derive(Debug, PartialEq, Eq)]
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
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = FieldMatcher::new("003@?")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_field_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidFieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        fields: impl IntoIterator<Item = &'a Field<T>> + Clone,
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

impl BitAnd for FieldMatcher {
    type Output = Self;

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

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Or,
            rhs: Box::new(rhs),
        }
    }
}

impl Not for FieldMatcher {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}

/// Parse field matcher singleton expression.
#[inline]
fn parse_field_matcher_singleton(
    i: &[u8],
) -> ParseResult<FieldMatcher> {
    map(parse_singleton_matcher, FieldMatcher::Singleton)(i)
}

/// Parse field matcher expression (curly bracket notation).
#[inline]
fn parse_field_matcher_singleton_bracket(
    i: &[u8],
) -> ParseResult<FieldMatcher> {
    map(parse_singleton_matcher_bracket, FieldMatcher::Singleton)(i)
}

/// Parse field matcher exists expression.
#[inline]
fn parse_field_matcher_exists(i: &[u8]) -> ParseResult<FieldMatcher> {
    alt((
        map(parse_exists_matcher, |matcher| {
            FieldMatcher::Singleton(SingletonMatcher::Exists(matcher))
        }),
        map(
            tuple((
                parse_tag_matcher,
                parse_occurrence_matcher,
                preceded(
                    ws(char('.')),
                    subfield_matcher::parse_exists_matcher,
                ),
            )),
            |(t, o, s)| {
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
            },
        ),
    ))(i)
}

/// Parse field matcher cardinality expression.
#[inline]
fn parse_field_matcher_cardinality(
    i: &[u8],
) -> ParseResult<FieldMatcher> {
    map(parse_cardinality_matcher, FieldMatcher::Cardinality)(i)
}

/// Parse a field matcher group expression.
fn parse_field_matcher_group(i: &[u8]) -> ParseResult<FieldMatcher> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(
                alt((
                    parse_field_matcher_composite,
                    parse_field_matcher_singleton,
                    parse_field_matcher_not,
                    parse_field_matcher_cardinality,
                    parse_field_matcher_group,
                )),
                ws(char(')')),
            )),
        ),
        |matcher| FieldMatcher::Group(Box::new(matcher)),
    )(i)
}

/// Parse a field matcher not expression.
fn parse_field_matcher_not(i: &[u8]) -> ParseResult<FieldMatcher> {
    map(
        preceded(
            ws(char('!')),
            cut(alt((
                parse_field_matcher_group,
                parse_field_matcher_singleton_bracket,
                parse_field_matcher_exists,
                parse_field_matcher_not,
            ))),
        ),
        |matcher| FieldMatcher::Not(Box::new(matcher)),
    )(i)
}

/// Parse a field matcher and expression.
fn parse_field_matcher_and(i: &[u8]) -> ParseResult<FieldMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_field_matcher_group),
            ws(parse_field_matcher_cardinality),
            ws(parse_field_matcher_singleton),
            ws(parse_field_matcher_not),
            ws(parse_field_matcher_exists),
        )),
        many1(preceded(
            ws(tag("&&")),
            alt((
                ws(parse_field_matcher_group),
                ws(parse_field_matcher_cardinality),
                ws(parse_field_matcher_singleton),
                ws(parse_field_matcher_not),
                ws(parse_field_matcher_exists),
            )),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev & next),
    ))
}

fn parse_field_matcher_or(i: &[u8]) -> ParseResult<FieldMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_field_matcher_group),
            ws(parse_field_matcher_and),
            ws(parse_field_matcher_cardinality),
            ws(parse_field_matcher_singleton),
            ws(parse_field_matcher_exists),
        )),
        many1(preceded(
            ws(tag("||")),
            cut(alt((
                ws(parse_field_matcher_group),
                ws(parse_field_matcher_and),
                ws(parse_field_matcher_cardinality),
                ws(parse_field_matcher_singleton),
                ws(parse_field_matcher_exists),
            ))),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev | next),
    ))
}

/// Parse a field matcher composite expression.
fn parse_field_matcher_composite(
    i: &[u8],
) -> ParseResult<FieldMatcher> {
    alt((parse_field_matcher_or, parse_field_matcher_and))(i)
}

/// Parse a field matcher expression.
pub fn parse_field_matcher(i: &[u8]) -> ParseResult<FieldMatcher> {
    alt((
        ws(parse_field_matcher_composite),
        ws(parse_field_matcher_group),
        ws(parse_field_matcher_not),
        ws(parse_field_matcher_singleton),
        ws(parse_field_matcher_cardinality),
    ))(i)
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::assert_finished_and_eq;
    use pica_record::Occurrence;

    use super::*;

    #[test]
    fn test_parse_exists_matcher() -> anyhow::Result<()> {
        assert_finished_and_eq!(
            parse_exists_matcher(b"003@?"),
            ExistsMatcher {
                tag_matcher: TagMatcher::new("003@")?,
                occurrence_matcher: OccurrenceMatcher::None,
            }
        );

        assert_finished_and_eq!(
            parse_exists_matcher(b"00[23]@?"),
            ExistsMatcher {
                tag_matcher: TagMatcher::new("00[23]@")?,
                occurrence_matcher: OccurrenceMatcher::None,
            }
        );

        assert_finished_and_eq!(
            parse_exists_matcher(b"012A/01?"),
            ExistsMatcher {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::Some(
                    Occurrence::new("01")
                ),
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfields_matcher() -> anyhow::Result<()> {
        assert_finished_and_eq!(
            parse_subfields_matcher(b"003@.0?"),
            SubfieldsMatcher {
                tag_matcher: TagMatcher::new("003@")?,
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: SubfieldMatcher::new("0?")?,
            }
        );

        assert_finished_and_eq!(
            parse_subfields_matcher(b"003@$0?"),
            SubfieldsMatcher {
                tag_matcher: TagMatcher::new("003@")?,
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: SubfieldMatcher::new("0?")?,
            }
        );

        assert_finished_and_eq!(
            parse_subfields_matcher(b"003@ $0?"),
            SubfieldsMatcher {
                tag_matcher: TagMatcher::new("003@")?,
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: SubfieldMatcher::new("0?")?,
            }
        );

        assert_finished_and_eq!(
            parse_subfields_matcher(b"003@{ #0 == 1 && 0? }"),
            SubfieldsMatcher {
                tag_matcher: TagMatcher::new("003@")?,
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: SubfieldMatcher::new(
                    "#0 == 1 && 0?"
                )?,
            }
        );

        Ok(())
    }
}
