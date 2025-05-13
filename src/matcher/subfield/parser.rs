use std::cell::RefCell;

use bstr::ByteSlice;
use either::Either::Left;
use either::Right;
use regex::bytes::Regex;
use winnow::ascii::{digit1, multispace1};
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, separated, terminated,
};
use winnow::error::ParserError;
use winnow::prelude::*;

use super::{
    CardinalityMatcher, ContainsMatcher, ExistsMatcher, InMatcher,
    RegexMatcher, RegexSetMatcher, RelationMatcher, SingletonMatcher,
    SubfieldMatcher,
};
use crate::matcher::operator::{
    RelationalOp, parse_relational_operator,
};
use crate::matcher::quantifier::parse_quantifier;
use crate::parser::{parse_string, parse_subfield_codes, ws};
use crate::primitives::parse::parse_subfield_code;

/// Parses a [ExistsMatcher] expression.
pub(crate) fn parse_exists_matcher(
    i: &mut &[u8],
) -> ModalResult<ExistsMatcher> {
    terminated(parse_subfield_codes, '?')
        .with_taken()
        .map(|(codes, raw_data)| {
            let raw_data = raw_data.to_str().unwrap().to_string();
            ExistsMatcher { raw_data, codes }
        })
        .parse_next(i)
}

/// Parse a [RelationMatcher] expression.
#[inline]
pub(crate) fn parse_relation_matcher(
    i: &mut &[u8],
) -> ModalResult<RelationMatcher> {
    (
        opt(ws(terminated(parse_quantifier, multispace1)))
            .map(Option::unwrap_or_default),
        alt((
            ws(parse_subfield_codes),
            #[cfg(feature = "compat")]
            preceded(
                ws('$'),
                ws(crate::parser::parse_subfield_codes_compat),
            ),
        )),
        ws(parse_relational_operator)
            .verify(RelationalOp::is_str_applicable),
        ws(parse_string),
    )
        .with_taken()
        .map(|((quantifier, codes, op, value), raw_data)| {
            let raw_data =
                raw_data.to_str().unwrap().trim().to_string();

            RelationMatcher {
                quantifier,
                codes,
                op,
                value,
                raw_data,
            }
        })
        .parse_next(i)
}

/// Parse a [ContainsMatcher] expression.
pub(crate) fn parse_contains_matcher(
    i: &mut &[u8],
) -> ModalResult<ContainsMatcher> {
    use std::collections::BTreeSet;

    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        alt((
            ws(parse_subfield_codes),
            #[cfg(feature = "compat")]
            preceded(
                ws('$'),
                ws(crate::parser::parse_subfield_codes_compat),
            ),
        )),
        ws("=?"),
        alt((
            delimited(
                ws('['),
                separated(1.., parse_string, ws(',')).map(
                    |values: Vec<Vec<u8>>| {
                        Right(BTreeSet::<Vec<u8>>::from_iter(values))
                    },
                ),
                ws(']'),
            ),
            ws(parse_string).map(Left),
        )),
    )
        .with_taken()
        .map(|((quantifier, codes, _, values), raw_data)| {
            let raw_data = raw_data.to_str().unwrap().to_string();
            ContainsMatcher {
                quantifier,
                codes,
                values,
                raw_data,
            }
        })
        .parse_next(i)
}

/// Parse a [RegexMatcher] expression.
pub(crate) fn parse_regex_matcher(
    i: &mut &[u8],
) -> ModalResult<RegexMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        ws(parse_subfield_codes),
        ws(alt(("=~".value(false), "!~".value(true)))),
        parse_string
            .verify_map(|re| String::from_utf8(re).ok())
            .verify(|re| Regex::new(re).is_ok()),
    )
        .with_taken()
        .map(|((quantifier, codes, invert, re), raw_data)| {
            let raw_data = raw_data.to_str().unwrap().to_string();
            RegexMatcher {
                quantifier,
                codes,
                invert,
                regex: re,
                raw_data,
            }
        })
        .parse_next(i)
}

/// Parse a [RegexSetMatcher] expression.
pub(crate) fn parse_regex_set_matcher(
    i: &mut &[u8],
) -> ModalResult<RegexSetMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        ws(parse_subfield_codes),
        ws(alt(("=~".value(false), "!~".value(true)))),
        delimited(
            ws('['),
            separated(
                1..,
                parse_string
                    .verify_map(|re| String::from_utf8(re).ok())
                    .verify(|re| Regex::new(re).is_ok()),
                ws(','),
            ),
            ws(']'),
        ),
    )
        .with_taken()
        .map(|((quantifier, codes, invert, re), raw_data)| {
            let raw_data = raw_data.to_str().unwrap().to_string();
            RegexSetMatcher {
                quantifier,
                codes,
                invert,
                regex: re,
                raw_data,
            }
        })
        .parse_next(i)
}

/// Parse a [InMatcher] expression.
pub(crate) fn parse_in_matcher(
    i: &mut &[u8],
) -> ModalResult<InMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        ws(parse_subfield_codes),
        opt(ws("not")).map(|x| x.is_some()),
        preceded(
            ws("in"),
            alt((
                delimited(
                    ws('['),
                    separated(1.., parse_string, ws(',')),
                    ws(']'),
                ),
                parse_string.verify_map(|s| {
                    let values = s
                        .chars()
                        .map(|c| c.to_string().as_bytes().to_vec())
                        .collect::<Vec<Vec<u8>>>();

                    if values.len() > 1 { Some(values) } else { None }
                }),
            )),
        ),
    )
        .with_taken()
        .map(|((quantifier, codes, invert, values), raw_data)| {
            let raw_data = raw_data.to_str().unwrap().to_string();

            InMatcher {
                quantifier,
                codes,
                invert,
                values,
                raw_data,
            }
        })
        .parse_next(i)
}

/// Parse a [CardinalityMatcher] expression.
pub(crate) fn parse_cardinality_matcher(
    i: &mut &[u8],
) -> ModalResult<CardinalityMatcher> {
    preceded(
        ws('#'),
        (
            ws(parse_subfield_code),
            ws(parse_relational_operator)
                .verify(RelationalOp::is_usize_applicable),
            digit1
                .verify_map(|value| std::str::from_utf8(value).ok())
                .verify_map(|value| value.parse::<usize>().ok()),
        ),
    )
    .with_taken()
    .map(|((code, op, value), raw_data)| {
        let raw_data = raw_data.to_str().unwrap().to_string();

        CardinalityMatcher {
            code,
            op,
            value,
            raw_data,
        }
    })
    .parse_next(i)
}

/// Parse a singleton matcher expression.
pub(crate) fn parse_singleton_matcher(
    i: &mut &[u8],
) -> ModalResult<SingletonMatcher> {
    alt((
        parse_relation_matcher.map(SingletonMatcher::Relation),
        parse_contains_matcher.map(SingletonMatcher::Contains),
        parse_in_matcher.map(SingletonMatcher::In),
        parse_exists_matcher.map(SingletonMatcher::Exists),
        parse_cardinality_matcher.map(SingletonMatcher::Cardinality),
        parse_regex_set_matcher.map(SingletonMatcher::RegexSet),
        parse_regex_matcher.map(SingletonMatcher::Regex),
    ))
    .parse_next(i)
}

#[inline(always)]
pub(crate) fn parse_subfield_singleton_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    parse_singleton_matcher
        .map(SubfieldMatcher::Singleton)
        .parse_next(i)
}

thread_local! {
    pub static SUBFIELD_MATCHER_GROUP_LEVEL: RefCell<u32>
        = const { RefCell::new(0) };
}

fn group_level_reset() {
    SUBFIELD_MATCHER_GROUP_LEVEL.with(|level| *level.borrow_mut() = 0);
}

fn group_level_inc(i: &mut &[u8]) -> ModalResult<()> {
    SUBFIELD_MATCHER_GROUP_LEVEL.with(|level| {
        *level.borrow_mut() += 1;
        if *level.borrow() >= 256 {
            Err(winnow::error::ErrMode::from_input(i))
        } else {
            Ok(())
        }
    })
}

fn group_level_dec() {
    SUBFIELD_MATCHER_GROUP_LEVEL.with(|level| {
        *level.borrow_mut() -= 1;
    })
}

#[inline(always)]
fn parse_subfield_group_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    delimited(
        terminated(ws('('), group_level_inc),
        alt((
            parse_subfield_composite_matcher,
            parse_subfield_singleton_matcher,
            parse_subfield_not_matcher,
            parse_subfield_group_matcher,
        )),
        ws(')').map(|_| group_level_dec),
    )
    .map(|m| SubfieldMatcher::Group(Box::new(m)))
    .parse_next(i)
}

#[inline(always)]
fn parse_subfield_exists_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    parse_exists_matcher
        .map(SingletonMatcher::Exists)
        .map(SubfieldMatcher::Singleton)
        .parse_next(i)
}

#[inline(always)]
fn parse_subfield_not_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    preceded(
        ws('!'),
        alt((
            parse_subfield_group_matcher,
            parse_subfield_exists_matcher,
            parse_subfield_not_matcher,
        )),
    )
    .map(|matcher| SubfieldMatcher::Not(Box::new(matcher)))
    .parse_next(i)
}

fn parse_subfield_or_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    let atom = |i: &mut &[u8]| -> ModalResult<SubfieldMatcher> {
        ws(alt((
            parse_subfield_and_matcher,
            parse_subfield_xor_matcher,
            parse_subfield_group_matcher,
            parse_subfield_singleton_matcher,
            parse_subfield_not_matcher,
        )))
        .parse_next(i)
    };

    (atom, repeat(1.., preceded("||", atom)))
        .map(|(head, tail): (_, Vec<_>)| {
            tail.into_iter().fold(head, |prev, next| prev | next)
        })
        .parse_next(i)
}

fn parse_subfield_xor_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    let atom = |i: &mut &[u8]| -> ModalResult<SubfieldMatcher> {
        ws(alt((
            parse_subfield_and_matcher,
            parse_subfield_group_matcher,
            parse_subfield_singleton_matcher,
            parse_subfield_not_matcher,
        )))
        .parse_next(i)
    };

    (atom, repeat(1.., preceded(alt(("^", "XOR")), atom)))
        .map(|(head, tail): (_, Vec<_>)| {
            tail.into_iter().fold(head, |prev, next| prev ^ next)
        })
        .parse_next(i)
}

fn parse_subfield_and_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    let atom = |i: &mut &[u8]| -> ModalResult<SubfieldMatcher> {
        ws(alt((
            parse_subfield_group_matcher,
            parse_subfield_singleton_matcher,
            parse_subfield_not_matcher,
        )))
        .parse_next(i)
    };

    (atom, repeat(1.., preceded("&&", atom)))
        .map(|(head, tail): (_, Vec<_>)| {
            tail.into_iter().fold(head, |prev, next| prev & next)
        })
        .parse_next(i)
}

#[inline(always)]
fn parse_subfield_composite_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    alt((
        parse_subfield_or_matcher,
        parse_subfield_xor_matcher,
        parse_subfield_and_matcher,
    ))
    .parse_next(i)
}

pub(crate) fn parse_subfield_matcher(
    i: &mut &[u8],
) -> ModalResult<SubfieldMatcher> {
    alt((
        parse_subfield_composite_matcher,
        parse_subfield_singleton_matcher,
        parse_subfield_group_matcher,
        parse_subfield_not_matcher,
    ))
    .map(|matcher| {
        group_level_reset();
        matcher
    })
    .parse_next(i)
}

#[cfg(test)]
mod tests {
    use smallvec::SmallVec;

    use super::*;
    use crate::matcher::{BooleanOp, Quantifier};
    use crate::primitives::SubfieldCode;

    const SUBFIELD_CODES: &str = "0123456789\
        abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    #[test]
    fn test_parse_exists_matcher() {
        macro_rules! parse_success {
            ($input:expr, $codes:expr) => {
                assert_eq!(
                    parse_exists_matcher
                        .parse($input.as_bytes())
                        .unwrap(),
                    ExistsMatcher {
                        codes: SmallVec::from_iter(
                            $codes
                                .chars()
                                .map(SubfieldCode::from_unchecked)
                        ),
                        raw_data: $input.to_string(),
                    },
                );
            };
        }

        parse_success!("a?", "a");
        parse_success!("[ab]?", "ab");
        parse_success!("[abc]?", "abc");
        parse_success!("[a-c]?", "abc");
        parse_success!("[a-cd]?", "abcd");
        parse_success!("[a-cde]?", "abcde");
        parse_success!("[a-cde-f]?", "abcdef");
        parse_success!("[a-cde-fx]?", "abcdefx");
        parse_success!("*?", SUBFIELD_CODES);

        assert!(parse_exists_matcher.parse(b"=?").is_err());
        assert!(parse_exists_matcher.parse(b"=?").is_err());
        assert!(parse_exists_matcher.parse(b"[a-a]?").is_err());
        assert!(parse_exists_matcher.parse(b"[b-a]?").is_err());
        assert!(parse_exists_matcher.parse(b"[a-c=]?").is_err());
        assert!(parse_exists_matcher.parse(b"ALL a?").is_err());
        assert!(parse_exists_matcher.parse(b"ANY a?").is_err());
    }

    #[test]
    fn test_parse_relation_matcher() {
        use Quantifier::*;
        use RelationalOp::*;

        macro_rules! parse_success {
            ($i:expr, $q:expr, $codes:expr, $op:expr, $value:expr) => {
                let matcher = parse_relation_matcher
                    .parse($i.as_bytes())
                    .unwrap();
                assert_eq!(matcher.quantifier, $q);
                assert_eq!(
                    matcher.codes,
                    SmallVec::<[SubfieldCode; 4]>::from_iter(
                        $codes
                            .chars()
                            .map(SubfieldCode::from_unchecked)
                    )
                );
                assert_eq!(matcher.op, $op);
                assert_eq!(matcher.value, $value);
                assert_eq!(matcher.raw_data, $i);
            };
        }

        parse_success!("0 == 'abc'", Any, "0", Eq, b"abc");
        parse_success!("ALL 0 != 'abc'", All, "0", Ne, b"abc");
        parse_success!("ANY [ab] == 'abc'", Any, "ab", Eq, b"abc");
        parse_success!("a =^ 'abc'", Any, "a", StartsWith, b"abc");
        parse_success!("a !^ 'abc'", Any, "a", StartsNotWith, b"abc");
        parse_success!("a =$ 'abc'", Any, "a", EndsWith, b"abc");
        parse_success!("a !$ 'abc'", Any, "a", EndsNotWith, b"abc");
        parse_success!("a =* 'abc'", Any, "a", Similar, b"abc");
    }

    #[test]
    fn test_parse_contains_matcher() {
        use std::collections::BTreeSet;

        use Quantifier::*;

        macro_rules! parse_success {
            ($i:expr, $q:expr, $codes:expr, $rhs:expr) => {
                let matcher = parse_contains_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert_eq!(matcher.quantifier, $q);
                assert_eq!(matcher.raw_data, $i);
                assert_eq!(
                    matcher.codes,
                    SmallVec::<[SubfieldCode; 4]>::from_iter(
                        $codes
                            .chars()
                            .map(SubfieldCode::from_unchecked)
                    )
                );

                assert_eq!(matcher.values, $rhs);
            };
        }

        macro_rules! parse_success_r {
            ($i:expr, $q:expr, $codes:expr, $values:expr) => {
                parse_success!(
                    $i,
                    $q,
                    $codes,
                    Right(BTreeSet::from_iter(
                        $values
                            .iter()
                            .map(|value| value.as_bytes().to_vec())
                    ))
                )
            };
        }

        macro_rules! parse_success_l {
            ($i:expr, $q:expr, $codes:expr, $value:expr) => {
                parse_success!(
                    $i,
                    $q,
                    $codes,
                    Left($value.as_bytes().to_vec())
                )
            };
        }

        parse_success_r!("a =? ['foo']", Any, "a", vec!["foo"]);
        parse_success_l!("a =? 'foo'", Any, "a", "foo");

        parse_success_r!("ANY a =? ['foo']", Any, "a", vec!["foo"]);
        parse_success_l!("ANY a =? 'foo'", Any, "a", "foo");

        parse_success_r!("ALL a =? ['foo']", All, "a", vec!["foo"]);
        parse_success_l!("ALL a =? 'foo'", All, "a", "foo");

        parse_success_r!("[a-c] =? ['foo']", Any, "abc", vec!["foo"]);
        parse_success_l!("[a-c] =? 'foo'", Any, "abc", "foo");

        parse_success_r!("ANY [ab] =? ['foo']", Any, "ab", vec!["foo"]);
        parse_success_l!("ANY [ab] =? 'foo'", Any, "ab", "foo");

        parse_success_r!(
            "ALL [a-d] =? ['foo']",
            All,
            "abcd",
            vec!["foo"]
        );

        parse_success_l!("ALL [a-d] =? 'foo'", All, "abcd", "foo");

        parse_success_r!(
            "a =? ['foo','bar']",
            Any,
            "a",
            vec!["foo", "bar"]
        );
    }

    #[test]
    fn test_parse_regex_matcher() {
        use Quantifier::*;

        macro_rules! parse_success {
            ($i:expr, $q:expr, $codes:expr, $regex:expr, $invert:expr) => {
                assert_eq!(
                    parse_regex_matcher.parse($i.as_bytes()).unwrap(),
                    RegexMatcher {
                        quantifier: $q,
                        codes: SmallVec::<[SubfieldCode; 4]>::from_iter(
                            $codes
                                .chars()
                                .map(SubfieldCode::from_unchecked)
                        ),
                        invert: $invert,
                        regex: $regex.to_string(),
                        raw_data: $i.to_string(),
                    }
                );
            };
        }

        parse_success!("ALL [ab] =~ \"foo\"", All, "ab", "foo", false);
        parse_success!("[ab] =~ 'foo'", Any, "ab", "foo", false);
        parse_success!("ANY 0 =~ '^Tp'", Any, "0", "^Tp", false);
        parse_success!("ALL 0 =~ '^Tp'", All, "0", "^Tp", false);
        parse_success!("0 =~ '^Tp'", Any, "0", "^Tp", false);
        parse_success!("0 !~ '^Tp'", Any, "0", "^Tp", true);

        assert!(parse_regex_matcher.parse(b"0 =~ '[[ab]'").is_err());
        assert!(parse_regex_matcher.parse(b"0 !~ '[[ab]'").is_err());
    }

    #[test]
    fn test_parse_regex_set_matcher() {
        use Quantifier::*;

        macro_rules! parse_success {
            ($i:expr, $q:expr, $codes:expr, $rs:expr, $invert:expr) => {
                assert_eq!(
                    parse_regex_set_matcher
                        .parse($i.as_bytes())
                        .unwrap(),
                    RegexSetMatcher {
                        quantifier: $q,
                        codes: SmallVec::<[SubfieldCode; 4]>::from_iter(
                            $codes
                                .chars()
                                .map(SubfieldCode::from_unchecked)
                        ),
                        invert: $invert,
                        regex: $rs
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                        raw_data: $i.to_string(),
                    }
                );
            };
        }

        parse_success!(
            "ALL [ab] =~ [\"^foo\", \"bar$\"]",
            All,
            "ab",
            vec!["^foo", "bar$"],
            false
        );

        parse_success!(
            "[ab] =~ ['foo', 'bar']",
            Any,
            "ab",
            vec!["foo", "bar"],
            false
        );

        parse_success!(
            "ANY 0 =~ ['^Tp', '^Ts']",
            Any,
            "0",
            vec!["^Tp", "^Ts"],
            false
        );

        parse_success!(
            "ALL 0 =~ ['^Tp', '^Ts']",
            All,
            "0",
            vec!["^Tp", "^Ts"],
            false
        );

        parse_success!(
            "0 =~ ['^Tp', '^Ts']",
            Any,
            "0",
            vec!["^Tp", "^Ts"],
            false
        );

        parse_success!(
            "0 !~ ['^Tp', '^Ts']",
            Any,
            "0",
            vec!["^Tp", "^Ts"],
            true
        );

        assert!(
            parse_regex_matcher
                .parse(b"0 =~ ['[[ab]', 'Ts1']")
                .is_err()
        );

        assert!(
            parse_regex_matcher
                .parse(b"0 !~ ['Tp3', '[[ab]']")
                .is_err()
        );
    }

    #[test]
    fn test_parse_in_matcher() {
        use Quantifier::*;

        macro_rules! parse_success {
            ($i:expr, $q:expr, $codes:expr, $values:expr, $invert:expr) => {
                assert_eq!(
                    parse_in_matcher.parse($i.as_bytes()).unwrap(),
                    InMatcher {
                        quantifier: $q,
                        codes: SmallVec::<[SubfieldCode; 4]>::from_iter(
                            $codes
                                .chars()
                                .map(SubfieldCode::from_unchecked)
                        ),
                        invert: $invert,
                        values: $values
                            .iter()
                            .map(|item| item.as_bytes().to_vec())
                            .collect(),
                        raw_data: $i.to_string(),
                    }
                );
            };
        }

        parse_success!("0 in ['Tp1']", Any, "0", vec!["Tp1"], false);
        parse_success!(
            "0 in ['Tp1', 'Tpz']",
            Any,
            "0",
            vec!["Tp1", "Tpz"],
            false
        );

        parse_success!(
            "a not in 'adimnt'",
            Any,
            "a",
            vec!["a", "d", "i", "m", "n", "t"],
            true
        );

        parse_success!(
            "ANY [ab] in ['Tp1', 'Tpz']",
            Any,
            "ab",
            vec!["Tp1", "Tpz"],
            false
        );

        parse_success!(
            "ALL 0 in ['Tp1', 'Tpz']",
            All,
            "0",
            vec!["Tp1", "Tpz"],
            false
        );

        parse_success!(
            "0 not in ['Tp1', 'Tpz']",
            Any,
            "0",
            vec!["Tp1", "Tpz"],
            true
        );

        assert!(parse_in_matcher.parse(b"a in []").is_err());
        assert!(parse_in_matcher.parse(b"a in ''").is_err());
    }

    #[test]
    fn test_parse_cardinality_matcher() {
        use RelationalOp::*;

        macro_rules! parse_success {
            ($i:expr, $code:expr, $op:expr, $value:expr) => {
                assert_eq!(
                    parse_cardinality_matcher
                        .parse($i.as_bytes())
                        .unwrap(),
                    CardinalityMatcher {
                        code: SubfieldCode::from_unchecked($code),
                        op: $op,
                        value: $value,
                        raw_data: $i.to_string(),
                    }
                );
            };
        }

        parse_success!("#a == 1", 'a', Eq, 1);
        parse_success!("#a != 2", 'a', Ne, 2);
        parse_success!("#a >= 3", 'a', Ge, 3);
        parse_success!("#a > 4", 'a', Gt, 4);
        parse_success!("#a <= 5", 'a', Le, 5);
        parse_success!("#a < 6", 'a', Lt, 6);

        assert!(parse_cardinality_matcher.parse(b"#a > -1").is_err());
        assert!(parse_cardinality_matcher.parse(b"#a > '2'").is_err());
        assert!(parse_cardinality_matcher.parse(b"#a =^ 5").is_err());
        assert!(parse_cardinality_matcher.parse(b"#a !^ 5").is_err());
        assert!(parse_cardinality_matcher.parse(b"#a =$ 5").is_err());
        assert!(parse_cardinality_matcher.parse(b"#a !$ 5").is_err());
        assert!(parse_cardinality_matcher.parse(b"#a =* 5").is_err());
        assert!(parse_cardinality_matcher.parse(b"#a =? 5").is_err());
    }

    #[test]
    fn test_parse_singleton_matcher() {
        use Quantifier::*;
        use SingletonMatcher::*;

        macro_rules! parse_success {
            ($i:expr, $expected:pat) => {
                let o = parse_singleton_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert_eq!(o.to_string(), $i.to_string());
                assert!(matches!(o, $expected));
            };
        }

        parse_success!(
            "ALL a == 'foo'",
            Relation(RelationMatcher {
                quantifier: All,
                ..
            })
        );

        parse_success!("a?", Exists(ExistsMatcher { .. }));

        parse_success!(
            "a not in ['Tp1', 'Tsz']",
            In(InMatcher { invert: true, .. })
        );

        parse_success!(
            "#a > 3",
            Cardinality(CardinalityMatcher { value: 3, .. })
        );

        parse_success!(
            "a !~ '^Tp'",
            Regex(RegexMatcher { invert: true, .. })
        );

        parse_success!(
            "a =~ ['^Tp', '^Ts']",
            RegexSet(RegexSetMatcher { invert: false, .. })
        );
    }

    #[test]
    fn test_parse_subfield_singleton_matcher() {
        use SingletonMatcher::*;
        use SubfieldMatcher::*;

        macro_rules! parse_success {
            ($i:expr, $expected:pat) => {
                let o = parse_subfield_singleton_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert_eq!(o.to_string(), $i.to_string());
                assert!(matches!(o, $expected));
            };
        }

        parse_success!("a == 'foo'", Singleton(Relation(_)));
        parse_success!("a != 'foo'", Singleton(Relation(_)));
        parse_success!("a =^ 'foo'", Singleton(Relation(_)));
        parse_success!("a !^ 'foo'", Singleton(Relation(_)));
        parse_success!("a =$ 'foo'", Singleton(Relation(_)));
        parse_success!("a !$ 'foo'", Singleton(Relation(_)));
        parse_success!("a =* 'foo'", Singleton(Relation(_)));

        parse_success!("a =? ['foo', 'bar']", Singleton(Contains(_)));
        parse_success!("a =? 'foo'", Singleton(Contains(_)));

        parse_success!("#a == 1", Singleton(Cardinality(_)));
        parse_success!("#a != 1", Singleton(Cardinality(_)));
        parse_success!("#a >= 1", Singleton(Cardinality(_)));
        parse_success!("#a > 1", Singleton(Cardinality(_)));
        parse_success!("#a <= 1", Singleton(Cardinality(_)));
        parse_success!("#a < 1", Singleton(Cardinality(_)));

        parse_success!("a in ['foo', 'bar']", Singleton(In(_)));
        parse_success!("a not in ['foo', 'bar']", Singleton(In(_)));

        parse_success!("a =~ ['^Tp1', '^Ts']", Singleton(RegexSet(_)));
        parse_success!("a !~ ['^Tp1', '^Ts']", Singleton(RegexSet(_)));

        parse_success!("a =~ '^Tp1'", Singleton(Regex(_)));
        parse_success!("a !~ '^Tp1'", Singleton(Regex(_)));

        parse_success!("a?", Singleton(Exists(_)));
    }

    #[test]
    fn test_parse_subfield_not_matcher() {
        macro_rules! parse_success {
            ($i:expr) => {
                let o = parse_subfield_not_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert_eq!(o.to_string(), $i.to_string());
                assert!(matches!(o, SubfieldMatcher::Not(_)));
            };
        }

        parse_success!("!(a? && b == 'foo')");
        parse_success!("!!a?");
        parse_success!("!a?");

        assert!(
            parse_subfield_not_matcher.parse(b"!a == 'foo'").is_err()
        );
    }

    #[test]
    fn test_parse_subfield_group_matcher() {
        macro_rules! parse_success {
            ($i:expr) => {
                let o = parse_subfield_group_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert!(matches!(o, SubfieldMatcher::Group(_)));
                assert_eq!(o.to_string(), $i.to_string());
            };
        }

        parse_success!("(a? && b?)");
        parse_success!("(a == 'foo')");
        parse_success!("(!(a == 'foo'))");
        parse_success!("((a == 'foo'))");
    }

    #[test]
    fn test_parse_subfield_and_matcher() {
        macro_rules! parse_success {
            ($i:expr) => {
                let o = parse_subfield_and_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                eprintln!("o = {o:?}");

                assert_eq!(o.to_string(), $i.to_string());
                assert!(matches!(
                    o,
                    SubfieldMatcher::Composite {
                        op: BooleanOp::And,
                        ..
                    }
                ));
            };
        }

        parse_success!("a? && b?");
        parse_success!("a? && b? && c?");
        parse_success!("a? && b? && c? && d?");
        parse_success!("a? && (b?)");
        parse_success!("a? && (b? && c?)");
        parse_success!("(a? && b?) && c?");
        parse_success!("a? && (b? && (c? && d?))");
        parse_success!("a? && !b?");
        parse_success!("a? && !(b? && c?)");
        parse_success!("!(a? && !(b? && c?)) && d?");

        assert!(
            parse_subfield_and_matcher
                .parse(b"a? && b? || c?")
                .is_err()
        );

        assert!(
            parse_subfield_and_matcher.parse(b"a? && b? ^ c?").is_err()
        );
    }

    #[test]
    fn test_parse_subfield_xor_matcher() {
        macro_rules! parse_success {
            ($i:expr) => {
                let o = parse_subfield_xor_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert_eq!(o.to_string(), $i.to_string());
                assert!(matches!(
                    o,
                    SubfieldMatcher::Composite {
                        op: BooleanOp::Xor,
                        ..
                    }
                ));
            };
        }

        parse_success!("a? ^ b?");
        parse_success!("a? ^ b? ^ c?");
        parse_success!("a? ^ b? ^ c? ^ d?");
        parse_success!("!a? ^ b? ^ c? ^ d?");
        parse_success!("!a? ^ !b? ^ c? ^ d?");
        parse_success!("!a? ^ !b? ^ !c? ^ d?");
        parse_success!("!a? ^ !b? ^ !c? ^ !d?");
        parse_success!("a? ^ b? && c? && d?");
        parse_success!("a? && b? ^ c? && d?");
        parse_success!("a? && b? && c? ^ d?");
        parse_success!("a? ^ (b? ^ c?) ^ d?");
        parse_success!("a? ^ ((b? ^ c?) ^ d?)");

        assert!(
            parse_subfield_xor_matcher.parse(b"a? ^ b? || c?").is_err()
        );
    }

    #[test]
    fn test_parse_subfield_or_matcher() {
        macro_rules! parse_success {
            ($i:expr) => {
                let o = parse_subfield_or_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert_eq!(o.to_string(), $i.to_string());
                assert!(matches!(
                    o,
                    SubfieldMatcher::Composite {
                        op: BooleanOp::Or,
                        ..
                    }
                ));
            };
        }

        parse_success!("a? || b?");
        parse_success!("a? || b? || c?");
        parse_success!("a? || b? || c? || d?");
        parse_success!("a? || !b?");
        parse_success!("a? || !(b? || c?)");
        parse_success!("!a? || !(b? || c?)");
        parse_success!("a? || b? && c?");
        parse_success!("a? && b? || c? && d?");
        parse_success!("a? || b? ^ c?");
        parse_success!("a? ^ b? || c? ^ d?");
        parse_success!("a? || (b? && c?)");
        parse_success!("(a? && b?) || (c? ^ d?)");
    }

    #[test]
    fn test_parse_subfield_matcher() {
        macro_rules! parse_success {
            ($i:expr) => {
                let o = parse_subfield_matcher
                    .parse($i.as_bytes())
                    .unwrap();

                assert_eq!(o.to_string(), $i.to_string());
            };
        }

        parse_success!("(a? && b?) || (c == 'foo' ^ d == 'bar')");
        parse_success!("0 in [\"Olfo\",\"Oaf\",\"Oa\"]");
        parse_success!("0 =^ \"A\" || 0 =^ \"O\"");
        parse_success!("c == '01' && d == 'MVB'");

        parse_success!(
            "p =~ '^\\\\[publtype\\\\]([bB]ookPart|[cC]onferencePaper)$'"
        );

        parse_success!(
            "a =~ '(?i)(Zeitschrift|Journal)' && \
                a !~ '(?i)(Abdr|Sonderdr|Diss(\\\\.|ertation)|Teile)'"
        );
    }
}
