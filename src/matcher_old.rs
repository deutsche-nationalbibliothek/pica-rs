use std::ops::{BitAnd, BitOr};
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{all_consuming, cut, map, map_res, opt};
use nom::multi::many0;
use nom::sequence::{preceded, terminated, tuple};
use nom::Finish;

use crate::common::{ws, ParseResult};
use crate::matcher::{
    parse_comparison_op_usize, parse_field_matcher, parse_occurrence_matcher,
    parse_subfield_list_matcher, parse_tag_matcher, BooleanOp, ComparisonOp,
    FieldMatcher, MatcherFlags, OccurrenceMatcher, SubfieldListMatcher, TagMatcher,
};
use crate::ByteRecord;

#[derive(Debug, PartialEq)]
pub enum RecordMatcher {
    Singleton(Box<FieldMatcher>),
    Group(Box<RecordMatcher>),
    Not(Box<RecordMatcher>),
    Composite(Box<RecordMatcher>, BooleanOp, Box<RecordMatcher>),
    Cardinality(
        TagMatcher,
        OccurrenceMatcher,
        Option<Box<SubfieldListMatcher>>,
        ComparisonOp,
        usize,
    ),
    True,
}

impl RecordMatcher {
    pub fn is_match(&self, record: &ByteRecord, flags: &MatcherFlags) -> bool {
        match self {
            Self::Singleton(matcher) => {
                record.iter().any(|field| matcher.is_match(field, flags))
            }
            Self::Group(matcher) => matcher.is_match(record, flags),
            Self::Not(matcher) => !matcher.is_match(record, flags),
            Self::Composite(lhs, BooleanOp::And, rhs) => {
                lhs.is_match(record, flags) && rhs.is_match(record, flags)
            }
            Self::Composite(lhs, BooleanOp::Or, rhs) => {
                lhs.is_match(record, flags) || rhs.is_match(record, flags)
            }
            Self::Cardinality(tag, occurrence, subfields, op, value) => {
                let fields = record
                    .iter()
                    .filter(|field| {
                        tag.is_match(field.tag())
                            && occurrence.is_match(field.occurrence())
                    })
                    .filter(|field| {
                        if let Some(matcher) = subfields {
                            matcher.is_match(field.subfields(), flags)
                        } else {
                            true
                        }
                    });

                let cardinality = fields.count();

                match op {
                    ComparisonOp::Eq => cardinality == *value,
                    ComparisonOp::Ne => cardinality != *value,
                    ComparisonOp::Gt => cardinality > *value,
                    ComparisonOp::Ge => cardinality >= *value,
                    ComparisonOp::Lt => cardinality < *value,
                    ComparisonOp::Le => cardinality <= *value,
                    _ => unreachable!(),
                }
            }
            Self::True => true,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseRecordMatcherError(String);

impl std::fmt::Display for ParseRecordMatcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseRecordMatcherError {}

impl FromStr for RecordMatcher {
    type Err = ParseRecordMatcherError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match all_consuming(parse_record_matcher)(s.as_bytes()).finish() {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(ParseRecordMatcherError(format!(
                "expected valid matcher expression, got '{}'",
                s
            ))),
        }
    }
}

impl BitAnd for RecordMatcher {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        RecordMatcher::Composite(Box::new(self), BooleanOp::And, Box::new(rhs))
    }
}

impl BitOr for RecordMatcher {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        RecordMatcher::Composite(Box::new(self), BooleanOp::Or, Box::new(rhs))
    }
}

fn parse_record_matcher_singleton(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(ws(parse_field_matcher), |x| {
        RecordMatcher::Singleton(Box::new(x))
    })(i)
}

fn parse_record_matcher_group(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(
                alt((
                    parse_record_matcher_composite,
                    parse_record_matcher_singleton,
                    parse_record_matcher_not,
                    parse_record_matcher_group,
                )),
                ws(char(')')),
            )),
        ),
        |matcher| RecordMatcher::Group(Box::new(matcher)),
    )(i)
}

fn parse_record_matcher_not(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(
        preceded(
            ws(char('!')),
            cut(alt((
                parse_record_matcher_group,
                parse_record_matcher_singleton,
                parse_record_matcher_not,
            ))),
        ),
        |matcher| RecordMatcher::Not(Box::new(matcher)),
    )(i)
}

fn parse_record_matcher_composite_and(i: &[u8]) -> ParseResult<RecordMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_record_matcher_group),
            ws(parse_record_matcher_singleton),
            ws(parse_record_matcher_not),
        )),
        many0(preceded(
            ws(tag("&&")),
            alt((
                ws(parse_record_matcher_group),
                ws(parse_record_matcher_singleton),
                ws(parse_record_matcher_not),
            )),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev & next),
    ))
}

fn parse_record_matcher_composite_or(i: &[u8]) -> ParseResult<RecordMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_record_matcher_group),
            ws(parse_record_matcher_composite_and),
            ws(parse_record_matcher_singleton),
            ws(parse_record_matcher_not),
        )),
        many0(preceded(
            ws(tag("||")),
            cut(alt((
                ws(parse_record_matcher_group),
                ws(parse_record_matcher_composite_and),
                ws(parse_record_matcher_singleton),
                ws(parse_record_matcher_not),
            ))),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev | next),
    ))
}

fn parse_record_matcher_composite(i: &[u8]) -> ParseResult<RecordMatcher> {
    alt((
        parse_record_matcher_composite_or,
        parse_record_matcher_composite_and,
    ))(i)
}

fn parse_record_matcher_cardinality(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(
        preceded(
            ws(char('#')),
            cut(tuple((
                ws(parse_tag_matcher),
                parse_occurrence_matcher,
                opt(preceded(
                    ws(char('{')),
                    cut(terminated(parse_subfield_list_matcher, ws(char('}')))),
                )),
                ws(parse_comparison_op_usize),
                map_res(digit1, |s| std::str::from_utf8(s).unwrap().parse::<usize>()),
            ))),
        ),
        |(t, o, s, op, value)| {
            RecordMatcher::Cardinality(t, o, s.map(Box::new), op, value)
        },
    )(i)
}

fn parse_record_matcher(i: &[u8]) -> ParseResult<RecordMatcher> {
    alt((
        ws(parse_record_matcher_group),
        ws(parse_record_matcher_not),
        ws(parse_record_matcher_composite),
        ws(parse_record_matcher_singleton),
        ws(parse_record_matcher_cardinality),
    ))(i)
}
