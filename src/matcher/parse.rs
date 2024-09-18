use std::cell::RefCell;

use bstr::ByteSlice;
use regex::bytes::Regex;
use smallvec::SmallVec;
use winnow::ascii::{digit1, multispace0, multispace1};
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, separated, separated_pair,
    terminated,
};
use winnow::error::ParserError;
use winnow::prelude::*;
use winnow::stream::{AsChar, Stream, StreamIsPartial};

use super::string::parse_string;
use super::subfield::{
    ExistsMatcher, RegexSetMatcher, SubfieldMatcher,
};
use super::{
    CardinalityMatcher, InMatcher, Quantifier, RegexMatcher,
    RelationMatcher, RelationalOp, SingletonMatcher,
};
use crate::primitives::parse::parse_subfield_code;
use crate::primitives::SubfieldCode;

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<I, O, E: ParserError<I>, F>(
    mut inner: F,
) -> impl Parser<I, O, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar + Clone,
    F: Parser<I, O, E>,
{
    move |i: &mut I| {
        let _ = multispace0.parse_next(i)?;
        let o = inner.parse_next(i);
        let _ = multispace0.parse_next(i)?;
        o
    }
}

#[inline]
pub(crate) fn parse_subfield_code_range(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    separated_pair(parse_subfield_code, b'-', parse_subfield_code)
        .verify(|(min, max)| min < max)
        .map(|(min, max)| {
            (min.as_byte()..=max.as_byte())
                .map(SubfieldCode::from_unchecked)
                .collect()
        })
        .parse_next(i)
}

#[inline]
fn parse_subfield_code_list(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    delimited(
        '[',
        repeat(
            1..,
            alt((
                parse_subfield_code_range,
                parse_subfield_code.map(|code| vec![code]),
            )),
        )
        .fold(Vec::new, |mut acc: Vec<_>, item| {
            acc.extend_from_slice(&item);
            acc
        }),
        ']',
    )
    .parse_next(i)
}

#[inline]
fn parse_subfield_code_all(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    const SUBFIELD_CODES: &[u8; 62] = b"0123456789\
        abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    '*'.value(
        SUBFIELD_CODES
            .into_iter()
            .map(|code| SubfieldCode::from_unchecked(*code))
            .collect(),
    )
    .parse_next(i)
}

/// Parse a list of subfield codes
#[allow(dead_code)]
pub(crate) fn parse_subfield_codes(
    i: &mut &[u8],
) -> PResult<SmallVec<[SubfieldCode; 4]>> {
    alt((
        parse_subfield_code_list,
        parse_subfield_code.map(|code| vec![code]),
        parse_subfield_code_all,
    ))
    .map(SmallVec::from_vec)
    .parse_next(i)
}

/// Parse the matcher expression from a byte slice.
pub(crate) fn parse_exists_matcher(
    i: &mut &[u8],
) -> PResult<ExistsMatcher> {
    terminated(parse_subfield_codes, '?')
        .with_taken()
        .map(|(codes, raw_data)| ExistsMatcher {
            raw_data: raw_data.to_str().unwrap().to_string(),
            codes,
        })
        .parse_next(i)
}

#[inline]
pub(crate) fn parse_quantifier(i: &mut &[u8]) -> PResult<Quantifier> {
    alt(("ALL".value(Quantifier::All), "ANY".value(Quantifier::Any)))
        .parse_next(i)
}

/// Parse RelationalOp which can be used for string comparisons.
#[inline]
pub(crate) fn parse_relational_operator(
    i: &mut &[u8],
) -> PResult<RelationalOp> {
    alt((
        "==".value(RelationalOp::Equal),
        "!=".value(RelationalOp::NotEqual),
        "=^".value(RelationalOp::StartsWith),
        "!^".value(RelationalOp::StartsNotWith),
        "=$".value(RelationalOp::EndsWith),
        "!$".value(RelationalOp::EndsNotWith),
        "=*".value(RelationalOp::Similar),
        "=?".value(RelationalOp::Contains),
        ">=".value(RelationalOp::GreaterThanOrEqual),
        ">".value(RelationalOp::GreaterThan),
        "<=".value(RelationalOp::LessThanOrEqual),
        "<".value(RelationalOp::LessThan),
    ))
    .parse_next(i)
}

/// Parse a relational expression
#[inline]
pub(crate) fn parse_relation_matcher(
    i: &mut &[u8],
) -> PResult<RelationMatcher> {
    (
        opt(ws(terminated(parse_quantifier, multispace1)))
            .map(Option::unwrap_or_default),
        ws(parse_subfield_codes),
        ws(parse_relational_operator)
            .verify(RelationalOp::is_str_applicable),
        ws(parse_string),
    )
        .with_taken()
        .map(|((quantifier, codes, op, value), raw_data)| {
            RelationMatcher {
                quantifier,
                codes,
                op,
                value,
                raw_data: raw_data.to_str().unwrap().to_string(),
            }
        })
        .parse_next(i)
}

pub(crate) fn parse_regex_matcher(
    i: &mut &[u8],
) -> PResult<RegexMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        ws(parse_subfield_codes),
        ws(alt(("=~".value(false), "!~".value(true)))),
        parse_string
            .verify_map(|re| String::from_utf8(re).ok())
            .verify(|re| Regex::new(&re).is_ok()),
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

pub(crate) fn parse_regex_set_matcher(
    i: &mut &[u8],
) -> PResult<RegexSetMatcher> {
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
                    .verify(|re| Regex::new(&re).is_ok()),
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

/// Parse a in matcher expression.
pub(crate) fn parse_in_matcher(i: &mut &[u8]) -> PResult<InMatcher> {
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

                    if values.len() > 1 {
                        Some(values)
                    } else {
                        None
                    }
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

/// Parse a cardinality matcher expression.
pub(crate) fn parse_cardinality_matcher(
    i: &mut &[u8],
) -> PResult<CardinalityMatcher> {
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
) -> PResult<SingletonMatcher> {
    alt((
        parse_relation_matcher.map(SingletonMatcher::Relation),
        parse_in_matcher.map(SingletonMatcher::In),
        parse_exists_matcher.map(SingletonMatcher::Exists),
        parse_cardinality_matcher.map(SingletonMatcher::Cardinality),
        parse_regex_set_matcher.map(SingletonMatcher::RegexSet),
        parse_regex_matcher.map(SingletonMatcher::Regex),
    ))
    .parse_next(i)
}

#[inline(always)]
fn parse_subfield_singleton_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
    parse_singleton_matcher
        .map(SubfieldMatcher::Singleton)
        .parse_next(i)
}

thread_local! {
    pub static SUBFIELD_MATCHER_GROUP_LEVEL: RefCell<u32>
        = const { RefCell::new(0) };
}

fn group_level_inc(i: &mut &[u8]) -> PResult<()> {
    SUBFIELD_MATCHER_GROUP_LEVEL.with(|level| {
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

fn group_level_dec() {
    SUBFIELD_MATCHER_GROUP_LEVEL.with(|level| {
        *level.borrow_mut() -= 1;
    })
}

#[inline(always)]
fn parse_subfield_group_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
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
) -> PResult<SubfieldMatcher> {
    parse_exists_matcher
        .map(SingletonMatcher::Exists)
        .map(SubfieldMatcher::Singleton)
        .parse_next(i)
}

#[inline(always)]
fn parse_subfield_not_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
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
) -> PResult<SubfieldMatcher> {
    let atom = |i: &mut &[u8]| -> PResult<SubfieldMatcher> {
        ws(alt((
            parse_subfield_group_matcher,
            parse_subfield_xor_matcher,
            parse_subfield_and_matcher,
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
) -> PResult<SubfieldMatcher> {
    let atom = |i: &mut &[u8]| -> PResult<SubfieldMatcher> {
        ws(alt((
            parse_subfield_group_matcher,
            parse_subfield_and_matcher,
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
) -> PResult<SubfieldMatcher> {
    let atom = |i: &mut &[u8]| -> PResult<SubfieldMatcher> {
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
) -> PResult<SubfieldMatcher> {
    alt((
        parse_subfield_or_matcher,
        parse_subfield_xor_matcher,
        parse_subfield_and_matcher,
    ))
    .parse_next(i)
}

pub(crate) fn parse_subfield_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
    alt((
        parse_subfield_composite_matcher,
        parse_subfield_group_matcher,
        parse_subfield_not_matcher,
        parse_subfield_singleton_matcher,
    ))
    .parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUBFIELD_CODES: &str = "0123456789\
        abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    #[test]
    fn test_parse_subfield_code_range() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_subfield_code_range
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                        .into_iter()
                        .map(SubfieldCode::from_unchecked)
                        .collect::<Vec<_>>()
                );
            };
        }

        parse_success!("a-b", ['a', 'b']);
        parse_success!("a-c", ['a', 'b', 'c']);
        parse_success!("a-z", ('a'..='z'));
        parse_success!("0-9", ('0'..='9'));
        parse_success!("A-Z", ('A'..='Z'));

        assert!(parse_subfield_code_range.parse(b"a-a").is_err());
        assert!(parse_subfield_code_range.parse(b"a-!").is_err());
        assert!(parse_subfield_code_range.parse(b"c-a").is_err());
    }

    #[test]
    fn test_parse_subfield_code_list() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_subfield_code_list
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                        .into_iter()
                        .map(SubfieldCode::from_unchecked)
                        .collect::<Vec<_>>()
                );
            };
        }

        parse_success!("[ab]", ['a', 'b']);
        parse_success!("[abc]", ['a', 'b', 'c']);
        parse_success!("[a-z]", ('a'..='z'));
        parse_success!("[0-9]", ('0'..='9'));
        parse_success!("[A-Z]", ('A'..='Z'));
        parse_success!("[0a-cz]", ['0', 'a', 'b', 'c', 'z']);

        assert!(parse_subfield_code_range.parse(b"[ab!]").is_err());
        assert!(parse_subfield_code_range.parse(b"[a-a]").is_err());
        assert!(parse_subfield_code_range.parse(b"[a-!]").is_err());
        assert!(parse_subfield_code_range.parse(b"[c-a]").is_err());
    }

    #[test]
    fn test_parse_subfield_code_all() {
        assert_eq!(
            parse_subfield_code_all.parse(b"*").unwrap(),
            SUBFIELD_CODES
                .chars()
                .map(SubfieldCode::from_unchecked)
                .collect::<Vec<_>>()
        );
    }

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
    fn test_parse_quantifier() {
        assert_eq!(
            parse_quantifier.parse(b"ALL").unwrap(),
            Quantifier::All
        );

        assert_eq!(
            parse_quantifier.parse(b"ANY").unwrap(),
            Quantifier::Any
        );
    }

    #[test]
    fn test_parse_relational_operator() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_relational_operator
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                );
            };
        }

        parse_success!("==", RelationalOp::Equal);
        parse_success!("!=", RelationalOp::NotEqual);
        parse_success!(">=", RelationalOp::GreaterThanOrEqual);
        parse_success!(">", RelationalOp::GreaterThan);
        parse_success!("<=", RelationalOp::LessThanOrEqual);
        parse_success!("<", RelationalOp::LessThan);
        parse_success!("=^", RelationalOp::StartsWith);
        parse_success!("!^", RelationalOp::StartsNotWith);
        parse_success!("=$", RelationalOp::EndsWith);
        parse_success!("!$", RelationalOp::EndsNotWith);
        parse_success!("=*", RelationalOp::Similar);
        parse_success!("=?", RelationalOp::Contains);
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

        parse_success!("0 == 'abc'", Any, "0", Equal, b"abc");
        parse_success!("ALL 0 != 'abc'", All, "0", NotEqual, b"abc");
        parse_success!("ANY [ab] == 'abc'", Any, "ab", Equal, b"abc");
        parse_success!("a =^ 'abc'", Any, "a", StartsWith, b"abc");
        parse_success!("a !^ 'abc'", Any, "a", StartsNotWith, b"abc");
        parse_success!("a =$ 'abc'", Any, "a", EndsWith, b"abc");
        parse_success!("a !$ 'abc'", Any, "a", EndsNotWith, b"abc");
        parse_success!("a =* 'abc'", Any, "a", Similar, b"abc");
        parse_success!("a =? 'abc'", Any, "a", Contains, b"abc");
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

        assert!(parse_regex_matcher
            .parse(b"0 =~ ['[[ab]', 'Ts1']")
            .is_err());

        assert!(parse_regex_matcher
            .parse(b"0 !~ ['Tp3', '[[ab]']")
            .is_err());
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

        parse_success!("#a == 1", 'a', Equal, 1);
        parse_success!("#a != 2", 'a', NotEqual, 2);
        parse_success!("#a >= 3", 'a', GreaterThanOrEqual, 3);
        parse_success!("#a > 4", 'a', GreaterThan, 4);
        parse_success!("#a <= 5", 'a', LessThanOrEqual, 5);
        parse_success!("#a < 6", 'a', LessThan, 6);

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
        parse_success!("a =? 'foo'", Singleton(Relation(_)));

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

        assert!(parse_subfield_not_matcher
            .parse(b"!a == 'foo'")
            .is_err());
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

        assert!(parse_subfield_group_matcher
            .parse(
                b"((((((((((((((((((((((((((((((((a?\
                ))))))))))))))))))))))))))))))))"
            )
            .is_err());
    }
}
