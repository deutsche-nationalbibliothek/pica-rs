use std::cell::RefCell;

use bstr::ByteSlice;
use winnow::ascii::digit1;
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, terminated,
};
use winnow::error::ParserError;
use winnow::{PResult, Parser};

use super::{
    CardinalityMatcher, FieldMatcher, SingletonMatcher,
    SubfieldsMatcher,
};
use crate::matcher::field::ExistsMatcher;
use crate::matcher::occurrence::parse_occurrence_matcher;
use crate::matcher::operator::parse_relational_operator;
use crate::matcher::quantifier::parse_quantifier;
use crate::matcher::subfield::parser::{
    parse_subfield_matcher, parse_subfield_singleton_matcher,
};
use crate::matcher::tag::parse_tag_matcher;
use crate::matcher::{subfield, RelationalOp};
use crate::parser::ws;

pub(super) fn parse_exists_matcher(
    i: &mut &[u8],
) -> PResult<ExistsMatcher> {
    terminated(ws((parse_tag_matcher, parse_occurrence_matcher)), '?')
        .map(|(t, o)| ExistsMatcher {
            tag_matcher: t,
            occ_matcher: o,
        })
        .parse_next(i)
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
        .with_taken()
        .map(|((q, t, o, s), raw_data)| {
            let raw_data =
                raw_data.to_str().unwrap().trim_end().to_string();
            SubfieldsMatcher {
                quantifier: q,
                tag_matcher: t,
                occurrence_matcher: o,
                subfield_matcher: s,
                raw_data,
            }
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
        .with_taken()
        .map(|((q, t, o, s), raw_data)| {
            let raw_data = raw_data.to_str().unwrap().to_string();
            SubfieldsMatcher {
                quantifier: q,
                tag_matcher: t,
                occurrence_matcher: o,
                subfield_matcher: s,
                raw_data,
            }
        })
        .parse_next(i)
}

/// Parse a [SubfieldsMatcher] expression.
#[inline]
pub(super) fn parse_subfields_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldsMatcher> {
    alt((parse_subfields_matcher_dot, parse_subfields_matcher_bracket))
        .parse_next(i)
}

/// Parse a [SingletonMatcher] expression.
#[inline]
pub(super) fn parse_singleton_matcher(
    i: &mut &[u8],
) -> PResult<SingletonMatcher> {
    alt((
        parse_exists_matcher.map(SingletonMatcher::Exists),
        parse_subfields_matcher.map(SingletonMatcher::Subfields),
    ))
    .parse_next(i)
}

/// Parse a [CardinalityMatcher] expression.
pub(super) fn parse_cardinality_matcher(
    i: &mut &[u8],
) -> PResult<CardinalityMatcher> {
    preceded(
        ws('#'),
        (
            ws(parse_tag_matcher),
            ws(parse_occurrence_matcher),
            opt(delimited(ws('{'), parse_subfield_matcher, ws('}'))),
            ws(parse_relational_operator)
                .verify(RelationalOp::is_usize_applicable),
            digit1
                .verify_map(|value| std::str::from_utf8(value).ok())
                .verify_map(|value| value.parse::<usize>().ok()),
        ),
    )
    .with_taken()
    .map(|((t, o, s, op, value), raw_data)| {
        let raw_data = raw_data.to_str().unwrap().to_string();
        CardinalityMatcher {
            tag_matcher: t,
            occ_matcher: o,
            subfield_matcher: s,
            op,
            value,
            raw_data,
        }
    })
    .parse_next(i)
}

#[inline(always)]
fn parse_field_singleton_matcher(
    i: &mut &[u8],
) -> PResult<FieldMatcher> {
    parse_singleton_matcher
        .map(FieldMatcher::Singleton)
        .parse_next(i)
}

#[inline(always)]
fn parse_field_cardinality_matcher(
    i: &mut &[u8],
) -> PResult<FieldMatcher> {
    parse_cardinality_matcher
        .map(FieldMatcher::Cardinality)
        .parse_next(i)
}

#[inline]
fn parse_field_exists_matcher(i: &mut &[u8]) -> PResult<FieldMatcher> {
    alt((
        parse_exists_matcher.map(|matcher| {
            FieldMatcher::Singleton(SingletonMatcher::Exists(matcher))
        }),
        (
            opt(parse_quantifier).map(Option::unwrap_or_default),
            parse_tag_matcher,
            parse_occurrence_matcher,
            preceded(ws('.'), subfield::parser::parse_exists_matcher),
        )
            .with_taken()
            .map(|((q, t, o, s), raw_data)| {
                let raw_data = raw_data.to_str().unwrap().to_string();

                FieldMatcher::Singleton(SingletonMatcher::Subfields(
                    SubfieldsMatcher {
                        quantifier: q,
                        tag_matcher: t,
                        occurrence_matcher: o,
                        subfield_matcher:
                            subfield::SubfieldMatcher::Singleton(
                                subfield::SingletonMatcher::Exists(s),
                            ),
                        raw_data,
                    },
                ))
            }),
    ))
    .parse_next(i)
}

thread_local! {
    pub static FIELD_MATCHER_GROUP_LEVEL: RefCell<u32>
        = const { RefCell::new(0) };
}

fn group_level_reset() {
    FIELD_MATCHER_GROUP_LEVEL.with(|level| *level.borrow_mut() = 0);
}

fn group_level_inc(i: &mut &[u8]) -> PResult<()> {
    FIELD_MATCHER_GROUP_LEVEL.with(|level| {
        *level.borrow_mut() += 1;
        if *level.borrow() >= 256 {
            Err(winnow::error::ErrMode::from_error_kind(
                i,
                winnow::error::ErrorKind::Many,
            ))
        } else {
            Ok(())
        }
    })
}

fn group_level_dec() {
    FIELD_MATCHER_GROUP_LEVEL.with(|level| {
        *level.borrow_mut() -= 1;
    })
}

#[inline]
fn parse_field_group_matcher(i: &mut &[u8]) -> PResult<FieldMatcher> {
    delimited(
        terminated(ws('('), group_level_inc),
        alt((
            parse_field_composite_matcher,
            parse_field_singleton_matcher,
            parse_field_not_matcher,
            parse_field_cardinality_matcher,
            parse_field_group_matcher,
        )),
        ws(')').map(|_| group_level_dec()),
    )
    .map(|matcher| FieldMatcher::Group(Box::new(matcher)))
    .parse_next(i)
}

#[inline]
fn parse_field_not_matcher(i: &mut &[u8]) -> PResult<FieldMatcher> {
    preceded(
        ws('!'),
        alt((
            parse_field_group_matcher,
            parse_subfields_matcher_bracket
                .map(SingletonMatcher::Subfields)
                .map(FieldMatcher::Singleton),
            parse_field_exists_matcher,
            parse_field_not_matcher,
        )),
    )
    .map(|matcher| FieldMatcher::Not(Box::new(matcher)))
    .parse_next(i)
}

fn parse_field_or_matcher(i: &mut &[u8]) -> PResult<FieldMatcher> {
    let atom = |i: &mut &[u8]| -> PResult<FieldMatcher> {
        ws(alt((
            parse_field_and_matcher,
            parse_field_xor_matcher,
            parse_field_group_matcher,
            parse_field_cardinality_matcher,
            parse_field_singleton_matcher,
            parse_field_not_matcher,
            parse_field_exists_matcher,
        )))
        .parse_next(i)
    };

    (atom, repeat(1.., preceded(ws("||"), atom)))
        .map(|(head, tail): (_, Vec<_>)| {
            tail.into_iter().fold(head, |prev, next| prev | next)
        })
        .parse_next(i)
}

fn parse_field_xor_matcher(i: &mut &[u8]) -> PResult<FieldMatcher> {
    let atom = |i: &mut &[u8]| -> PResult<FieldMatcher> {
        ws(alt((
            parse_field_and_matcher,
            parse_field_group_matcher,
            parse_field_cardinality_matcher,
            parse_field_singleton_matcher,
            parse_field_not_matcher,
            parse_field_exists_matcher,
        )))
        .parse_next(i)
    };

    (atom, repeat(1.., preceded(alt(("^", "XOR")), atom)))
        .map(|(head, tail): (_, Vec<_>)| {
            tail.into_iter().fold(head, |prev, next| prev ^ next)
        })
        .parse_next(i)
}

fn parse_field_and_matcher(i: &mut &[u8]) -> PResult<FieldMatcher> {
    let atom = |i: &mut &[u8]| -> PResult<FieldMatcher> {
        ws(alt((
            parse_field_group_matcher,
            parse_field_cardinality_matcher,
            parse_field_singleton_matcher,
            parse_field_not_matcher,
            parse_field_exists_matcher,
        )))
        .parse_next(i)
    };

    (atom, repeat(1.., preceded(ws("&&"), atom)))
        .map(|(head, tail): (_, Vec<_>)| {
            tail.into_iter().fold(head, |prev, next| prev & next)
        })
        .parse_next(i)
}

#[inline(always)]
fn parse_field_composite_matcher(
    i: &mut &[u8],
) -> PResult<FieldMatcher> {
    alt((
        parse_field_or_matcher,
        parse_field_xor_matcher,
        parse_field_and_matcher,
    ))
    .parse_next(i)
}

/// Parse a [FieldMatcher] expression.
pub(crate) fn parse_field_matcher(
    i: &mut &[u8],
) -> PResult<FieldMatcher> {
    ws(alt((
        parse_field_composite_matcher,
        parse_field_group_matcher,
        parse_field_not_matcher,
        parse_field_singleton_matcher,
        parse_field_cardinality_matcher,
    )))
    .map(|matcher| {
        group_level_reset();
        matcher
    })
    .parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::{OccurrenceMatcher, Quantifier, TagMatcher};
    use crate::prelude::SubfieldMatcher;

    #[test]
    fn test_parse_exists_matcher() {
        macro_rules! parse_success {
            ($i:expr, $o:expr) => {
                let o =
                    parse_exists_matcher.parse($i.as_bytes()).unwrap();

                assert_eq!(o.to_string(), $i);
                assert_eq!(o, $o);
            };
        }

        parse_success!(
            "003@?",
            ExistsMatcher {
                tag_matcher: TagMatcher::new("003@").unwrap(),
                occ_matcher: OccurrenceMatcher::None
            }
        );

        parse_success!(
            "041A/*?",
            ExistsMatcher {
                tag_matcher: TagMatcher::new("041A").unwrap(),
                occ_matcher: OccurrenceMatcher::Any
            }
        );
    }

    #[test]
    fn test_parse_subfields_matcher() {
        macro_rules! parse_success {
            ($i:expr, $o:expr) => {
                let o = parse_subfields_matcher
                    .parse($i.as_bytes())
                    .unwrap();
                assert_eq!(o.to_string(), $i);
                assert_eq!(o, $o);
            };
        }

        parse_success!(
            "003@.0 == '0123456789X'",
            SubfieldsMatcher {
                raw_data: "003@.0 == '0123456789X'".to_string(),
                quantifier: Quantifier::Any,
                tag_matcher: TagMatcher::new("003@").unwrap(),
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: SubfieldMatcher::new(
                    "0 == '0123456789X'"
                )
                .unwrap(),
            }
        );

        parse_success!(
            "003@{0 == '0123456789X'}",
            SubfieldsMatcher {
                raw_data: "003@{0 == '0123456789X'}".to_string(),
                quantifier: Quantifier::Any,
                tag_matcher: TagMatcher::new("003@").unwrap(),
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: SubfieldMatcher::new(
                    "0 == '0123456789X'"
                )
                .unwrap(),
            }
        );
    }

    #[test]
    fn test_parse_singleton_matcher() {
        macro_rules! parse_success {
            ($i:expr, $o:expr) => {
                let o = parse_singleton_matcher
                    .parse($i.as_bytes())
                    .unwrap();
                assert_eq!(o.to_string(), $i);
                assert_eq!(o, $o);
            };
        }

        parse_success!(
            "041A/*?",
            SingletonMatcher::Exists(ExistsMatcher {
                tag_matcher: TagMatcher::new("041A").unwrap(),
                occ_matcher: OccurrenceMatcher::Any
            })
        );

        parse_success!(
            "003@.0 == '0123456789X'",
            SingletonMatcher::Subfields(SubfieldsMatcher {
                raw_data: "003@.0 == '0123456789X'".to_string(),
                quantifier: Quantifier::Any,
                tag_matcher: TagMatcher::new("003@").unwrap(),
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: SubfieldMatcher::new(
                    "0 == '0123456789X'"
                )
                .unwrap(),
            })
        );
    }

    #[test]
    fn test_parse_cardinality_matcher() {
        macro_rules! parse_success {
            ($i:expr, $o:expr) => {
                let o = parse_cardinality_matcher
                    .parse($i.as_bytes())
                    .unwrap();
                assert_eq!(o.to_string(), $i);
                assert_eq!(o, $o);
            };
        }

        parse_success!(
            "#010@ > 5",
            CardinalityMatcher {
                tag_matcher: TagMatcher::new("010@").unwrap(),
                occ_matcher: OccurrenceMatcher::None,
                subfield_matcher: None,
                op: RelationalOp::Gt,
                value: 5,
                raw_data: "#010@ > 5".to_string(),
            }
        );
    }
}
