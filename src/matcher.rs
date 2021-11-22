use std::ops::{BitAnd, BitOr, RangeFrom};
use std::str::FromStr;

use bstr::{BString, ByteSlice};
use regex::bytes::RegexBuilder;
use regex::Regex;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1, one_of};
use nom::combinator::{
    all_consuming, cut, map, map_res, opt, success, value, verify,
};
use nom::error::ParseError;
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};
use nom::{AsChar, FindToken, Finish, IResult, InputIter, InputLength, Slice};
use strsim::normalized_levenshtein;

use crate::common::{parse_string, ws, ParseResult};
use crate::occurrence::{parse_occurrence_digits, Occurrence};
use crate::subfield::{parse_subfield_code, Subfield};
use crate::tag::{parse_tag, Tag};
use crate::{ByteRecord, Field};

macro_rules! maybe_lowercase {
    ($value:expr, $flag:expr) => {
        if $flag {
            $value.to_lowercase()
        } else {
            $value
        }
    };
}

#[derive(Debug)]
pub struct MatcherFlags {
    pub ignore_case: bool,
    pub strsim_threshold: f64,
}

/// Comparison Operators
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    StartsWith,
    EndsWith,
    Similar,
}

/// Parses comparison operator for byte strings.
fn parse_comparison_op_bstring(i: &[u8]) -> ParseResult<ComparisonOp> {
    alt((
        value(ComparisonOp::Eq, tag("==")),
        value(ComparisonOp::Ne, tag("!=")),
        value(ComparisonOp::StartsWith, tag("=^")),
        value(ComparisonOp::EndsWith, tag("=$")),
        value(ComparisonOp::Similar, tag("=*")),
    ))(i)
}

/// Parses comparison operator for usize.
fn parse_comparison_op_usize(i: &[u8]) -> ParseResult<ComparisonOp> {
    alt((
        value(ComparisonOp::Eq, tag("==")),
        value(ComparisonOp::Ne, tag("!=")),
        value(ComparisonOp::Ge, tag(">=")),
        value(ComparisonOp::Gt, tag(">")),
        value(ComparisonOp::Le, tag("<=")),
        value(ComparisonOp::Lt, tag("<")),
    ))(i)
}

/// Boolean Operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BooleanOp {
    And,
    Or,
}

/// A subfield matcher.
#[derive(Debug, PartialEq)]
pub enum SubfieldMatcher {
    Comparison(Vec<char>, ComparisonOp, BString),
    Exists(Vec<char>),
    In(Vec<char>, Vec<BString>, bool),
    Regex(Vec<char>, String, bool),
}

impl SubfieldMatcher {
    /// Returns true if the `SubfieldMatcher` matches against the
    /// given `Subfield`, otherwise false.
    pub(crate) fn is_match(
        &self,
        subfield: &Subfield,
        flags: &MatcherFlags,
    ) -> bool {
        let case_cmp = |lhs: &BString, rhs: &BString| -> bool {
            if flags.ignore_case {
                lhs.to_lowercase() == rhs.to_lowercase()
            } else {
                lhs == rhs
            }
        };

        match self {
            Self::Comparison(codes, ComparisonOp::Eq, value) => {
                codes.contains(&subfield.code())
                    && case_cmp(subfield.value(), value)
            }
            Self::Comparison(codes, ComparisonOp::Ne, value) => {
                !codes.contains(&subfield.code())
                    || !case_cmp(subfield.value(), value)
            }
            Self::Comparison(codes, ComparisonOp::StartsWith, value) => {
                codes.contains(&subfield.code())
                    && if flags.ignore_case {
                        subfield
                            .value()
                            .to_lowercase()
                            .starts_with(&value.to_lowercase())
                    } else {
                        subfield.value().starts_with(value)
                    }
            }
            Self::Comparison(codes, ComparisonOp::EndsWith, value) => {
                codes.contains(&subfield.code())
                    && if flags.ignore_case {
                        subfield
                            .value()
                            .to_lowercase()
                            .ends_with(&value.to_lowercase())
                    } else {
                        subfield.value().ends_with(value)
                    }
            }
            Self::Comparison(codes, ComparisonOp::Similar, value) => {
                if codes.contains(&subfield.code()) {
                    let flag = flags.ignore_case;
                    let lhs = maybe_lowercase!(subfield.value().to_vec(), flag);
                    let rhs = maybe_lowercase!(value.to_vec(), flag);

                    let score = normalized_levenshtein(
                        &lhs.to_str_lossy(),
                        &rhs.to_str_lossy(),
                    );

                    score > flags.strsim_threshold
                } else {
                    false
                }
            }
            Self::Comparison(_, _, _) => unreachable!(),
            Self::Regex(codes, regex, invert) => {
                let re = RegexBuilder::new(regex)
                    .case_insensitive(flags.ignore_case)
                    .build()
                    .unwrap();

                let mut result = codes.contains(&subfield.code())
                    && re.is_match(subfield.value());

                if *invert {
                    result = !result;
                }

                result
            }
            Self::In(codes, values, invert) => {
                let mut result = codes.contains(&subfield.code())
                    && values
                        .iter()
                        .any(|x: &BString| case_cmp(subfield.value(), x));

                if *invert {
                    result = !result;
                }

                result
            }
            Self::Exists(codes) => codes.contains(&subfield.code()),
        }
    }
}

fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        preceded(
            char('['),
            cut(terminated(many1(parse_subfield_code), char(']'))),
        ),
        map(char('*'), |_| {
            "0123456789abcdefghijklmnopqrstuvwxyz"
                .chars()
                .collect::<Vec<char>>()
        }),
        map(parse_subfield_code, |x| vec![x]),
    ))(i)
}

fn parse_subfield_matcher_comparison(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        tuple((
            ws(parse_subfield_codes),
            ws(parse_comparison_op_bstring),
            ws(parse_string),
        )),
        |(codes, op, value)| {
            SubfieldMatcher::Comparison(codes, op, BString::from(value))
        },
    )(i)
}

fn parse_subfield_matcher_regex(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        tuple((
            parse_subfield_codes,
            alt((value(false, ws(tag("=~"))), value(true, ws(tag("!~"))))),
            verify(parse_string, |x| Regex::new(x).is_ok()),
        )),
        |(codes, invert, regex)| SubfieldMatcher::Regex(codes, regex, invert),
    )(i)
}
fn parse_subfield_matcher_in(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        tuple((
            parse_subfield_codes,
            opt(ws(tag("not"))),
            ws(tag("in")),
            preceded(
                ws(char('[')),
                cut(terminated(
                    separated_list1(
                        ws(char(',')),
                        map(parse_string, BString::from),
                    ),
                    ws(char(']')),
                )),
            ),
        )),
        |(codes, invert, _, values)| {
            SubfieldMatcher::In(codes, values, invert.is_some())
        },
    )(i)
}

fn parse_subfield_matcher_exists(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        terminated(ws(parse_subfield_codes), char('?')),
        SubfieldMatcher::Exists,
    )(i)
}

fn parse_subfield_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    alt((
        ws(parse_subfield_matcher_comparison),
        ws(parse_subfield_matcher_regex),
        ws(parse_subfield_matcher_in),
        ws(parse_subfield_matcher_exists),
    ))(i)
}

#[derive(Debug, PartialEq)]
pub enum SubfieldListMatcher {
    Singleton(SubfieldMatcher),
    Group(Box<SubfieldListMatcher>),
    Not(Box<SubfieldListMatcher>),
    Composite(
        Box<SubfieldListMatcher>,
        BooleanOp,
        Box<SubfieldListMatcher>,
    ),
    Cardinality(char, ComparisonOp, usize),
}

impl SubfieldListMatcher {
    pub fn is_match(
        &self,
        subfields: &[Subfield],
        flags: &MatcherFlags,
    ) -> bool {
        match self {
            Self::Singleton(matcher) => {
                subfields.iter().any(|s| matcher.is_match(s, flags))
            }
            Self::Group(matcher) => matcher.is_match(subfields, flags),
            Self::Not(matcher) => !matcher.is_match(subfields, flags),
            Self::Composite(lhs, BooleanOp::And, rhs) => {
                lhs.is_match(subfields, flags) && rhs.is_match(subfields, flags)
            }
            Self::Composite(lhs, BooleanOp::Or, rhs) => {
                lhs.is_match(subfields, flags) || rhs.is_match(subfields, flags)
            }
            Self::Cardinality(code, op, value) => {
                let cardinality =
                    subfields.iter().filter(|s| s.code() == *code).count();

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
        }
    }
}

impl BitAnd for SubfieldListMatcher {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        SubfieldListMatcher::Composite(
            Box::new(self),
            BooleanOp::And,
            Box::new(rhs),
        )
    }
}

impl BitOr for SubfieldListMatcher {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        SubfieldListMatcher::Composite(
            Box::new(self),
            BooleanOp::Or,
            Box::new(rhs),
        )
    }
}

fn parse_subfield_list_matcher_singleton(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    map(ws(parse_subfield_matcher), SubfieldListMatcher::Singleton)(i)
}

fn parse_subfield_list_matcher_cardinality(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    map(
        preceded(
            char('#'),
            cut(tuple((
                ws(parse_subfield_code),
                ws(parse_comparison_op_usize),
                map_res(digit1, |s| {
                    std::str::from_utf8(s).unwrap().parse::<usize>()
                }),
            ))),
        ),
        |(code, op, value)| SubfieldListMatcher::Cardinality(code, op, value),
    )(i)
}

fn parse_subfield_list_matcher_group(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(
                alt((
                    parse_subfield_list_matcher_composite,
                    parse_subfield_list_matcher_singleton,
                    parse_subfield_list_matcher_not,
                    parse_subfield_list_matcher_group,
                )),
                ws(char(')')),
            )),
        ),
        |matcher| SubfieldListMatcher::Group(Box::new(matcher)),
    )(i)
}

fn parse_subfield_list_matcher_not(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    map(
        preceded(
            ws(char('!')),
            cut(alt((
                parse_subfield_list_matcher_group,
                parse_subfield_list_matcher_singleton,
                parse_subfield_list_matcher_not,
            ))),
        ),
        |matcher| SubfieldListMatcher::Not(Box::new(matcher)),
    )(i)
}

fn parse_subfield_list_matcher_composite_and(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_subfield_list_matcher_group),
            ws(parse_subfield_list_matcher_singleton),
            ws(parse_subfield_list_matcher_not),
        )),
        many0(preceded(
            ws(tag("&&")),
            alt((
                ws(parse_subfield_list_matcher_group),
                ws(parse_subfield_list_matcher_singleton),
                ws(parse_subfield_list_matcher_not),
            )),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev & next),
    ))
}

fn parse_subfield_list_matcher_composite_or(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_subfield_list_matcher_group),
            ws(parse_subfield_list_matcher_composite_and),
            ws(parse_subfield_list_matcher_singleton),
            ws(parse_subfield_list_matcher_not),
        )),
        many0(preceded(
            ws(tag("||")),
            cut(alt((
                ws(parse_subfield_list_matcher_group),
                ws(parse_subfield_list_matcher_composite_and),
                ws(parse_subfield_list_matcher_singleton),
                ws(parse_subfield_list_matcher_not),
            ))),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev | next),
    ))
}

fn parse_subfield_list_matcher_composite(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    alt((
        parse_subfield_list_matcher_composite_or,
        parse_subfield_list_matcher_composite_and,
    ))(i)
}

pub(crate) fn parse_subfield_list_matcher(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    alt((
        parse_subfield_list_matcher_group,
        parse_subfield_list_matcher_not,
        parse_subfield_list_matcher_composite,
        parse_subfield_list_matcher_singleton,
        parse_subfield_list_matcher_cardinality,
    ))(i)
}

#[derive(Debug, PartialEq)]
pub enum TagMatcher {
    Some(Tag),
    Pattern(Vec<char>, Vec<char>, Vec<char>, Vec<char>),
}

impl TagMatcher {
    pub fn is_match(&self, tag: &Tag) -> bool {
        match self {
            TagMatcher::Some(lhs) => lhs == tag,
            TagMatcher::Pattern(p0, p1, p2, p3) => {
                p0.contains(&(tag[0] as char))
                    && p1.contains(&(tag[1] as char))
                    && p2.contains(&(tag[2] as char))
                    && p3.contains(&(tag[3] as char))
            }
        }
    }
}

fn parse_character_class<I, T, E: ParseError<I>>(
    list: T,
) -> impl FnMut(I) -> IResult<I, Vec<char>, E>
where
    I: Slice<RangeFrom<usize>> + InputIter + Clone + InputLength,
    <I as InputIter>::Item: AsChar + Copy,
    T: FindToken<<I as InputIter>::Item> + Clone,
{
    alt((
        preceded(
            char('['),
            cut(terminated(many1(one_of(list.clone())), char(']'))),
        ),
        map(one_of(list), |x| vec![x]),
    ))
}

pub(crate) fn parse_tag_matcher(i: &[u8]) -> ParseResult<TagMatcher> {
    alt((
        map(parse_tag, TagMatcher::Some),
        map(
            tuple((
                parse_character_class("012"),
                parse_character_class("0123456789"),
                parse_character_class("0123456789"),
                parse_character_class("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
            )),
            |(p1, p2, p3, p4)| TagMatcher::Pattern(p1, p2, p3, p4),
        ),
    ))(i)
}

#[derive(Debug, PartialEq, Clone)]
pub enum OccurrenceMatcher {
    Some(Occurrence),
    Range(Occurrence, Occurrence),
    Any,
    None,
}

impl OccurrenceMatcher {
    pub fn is_match(&self, occurrence: Option<&Occurrence>) -> bool {
        match occurrence {
            Some(occurrence) => match self {
                OccurrenceMatcher::Any => true,
                OccurrenceMatcher::None => occurrence == "00",
                OccurrenceMatcher::Some(rhs) => occurrence == rhs,
                OccurrenceMatcher::Range(min, max) => {
                    (occurrence >= min) && (occurrence <= max)
                }
            },
            None => {
                matches!(self, OccurrenceMatcher::Any | OccurrenceMatcher::None)
            }
        }
    }
}

pub(crate) fn parse_occurrence_matcher(
    i: &[u8],
) -> ParseResult<OccurrenceMatcher> {
    alt((
        preceded(
            char('/'),
            cut(alt((
                map(
                    verify(
                        separated_pair(
                            parse_occurrence_digits,
                            char('-'),
                            parse_occurrence_digits,
                        ),
                        |(min, max)| min.len() == max.len() && min < max,
                    ),
                    |(min, max)| {
                        OccurrenceMatcher::Range(
                            Occurrence::from_unchecked(min),
                            Occurrence::from_unchecked(max),
                        )
                    },
                ),
                map(
                    verify(parse_occurrence_digits, |x: &[u8]| x != b"00"),
                    |x| OccurrenceMatcher::Some(Occurrence::from_unchecked(x)),
                ),
                value(OccurrenceMatcher::None, tag("00")),
                value(OccurrenceMatcher::Any, char('*')),
            ))),
        ),
        success(OccurrenceMatcher::None),
    ))(i)
}

#[derive(Debug, PartialEq)]
pub enum FieldMatcher {
    Subield(TagMatcher, OccurrenceMatcher, SubfieldListMatcher),
    Exists(TagMatcher, OccurrenceMatcher),
}

impl FieldMatcher {
    pub fn is_match(&self, field: &Field, flags: &MatcherFlags) -> bool {
        match self {
            Self::Subield(tag, occurrence, subfield) => {
                tag.is_match(field.tag())
                    && occurrence.is_match(field.occurrence())
                    && subfield.is_match(field.subfields(), flags)
            }
            Self::Exists(tag, occurrence) => {
                tag.is_match(field.tag())
                    && occurrence.is_match(field.occurrence())
            }
        }
    }
}

fn parse_field_matcher_subfield(i: &[u8]) -> ParseResult<FieldMatcher> {
    map(
        tuple((
            parse_tag_matcher,
            parse_occurrence_matcher,
            alt((
                preceded(char('.'), cut(parse_subfield_list_matcher_singleton)),
                preceded(
                    ws(char('{')),
                    cut(terminated(parse_subfield_list_matcher, ws(char('}')))),
                ),
            )),
        )),
        |(tag, occurrence, subfields)| {
            FieldMatcher::Subield(tag, occurrence, subfields)
        },
    )(i)
}

fn parse_field_matcher_exists(i: &[u8]) -> ParseResult<FieldMatcher> {
    map(
        terminated(
            pair(ws(parse_tag_matcher), parse_occurrence_matcher),
            ws(char('?')),
        ),
        |(t, o)| FieldMatcher::Exists(t, o),
    )(i)
}

fn parse_field_matcher(i: &[u8]) -> ParseResult<FieldMatcher> {
    alt((parse_field_matcher_subfield, parse_field_matcher_exists))(i)
}

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
                map_res(digit1, |s| {
                    std::str::from_utf8(s).unwrap().parse::<usize>()
                }),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_comparison_op() -> TestResult {
        assert_eq!(parse_comparison_op_bstring(b"==")?.1, ComparisonOp::Eq);
        assert_eq!(parse_comparison_op_bstring(b"!=")?.1, ComparisonOp::Ne);
        assert_eq!(
            parse_comparison_op_bstring(b"=^")?.1,
            ComparisonOp::StartsWith
        );
        assert_eq!(
            parse_comparison_op_bstring(b"=$")?.1,
            ComparisonOp::EndsWith
        );

        assert_eq!(parse_comparison_op_usize(b"==")?.1, ComparisonOp::Eq);
        assert_eq!(parse_comparison_op_usize(b"!=")?.1, ComparisonOp::Ne);
        assert_eq!(parse_comparison_op_usize(b">=")?.1, ComparisonOp::Ge);
        assert_eq!(parse_comparison_op_usize(b">")?.1, ComparisonOp::Gt);
        assert_eq!(parse_comparison_op_usize(b"<=")?.1, ComparisonOp::Le);
        assert_eq!(parse_comparison_op_usize(b"<")?.1, ComparisonOp::Lt);

        Ok(())
    }

    #[test]
    fn test_parse_subfield_codes() -> TestResult {
        assert_eq!(parse_subfield_codes(b"[abc]")?.1, vec!['a', 'b', 'c']);
        assert_eq!(parse_subfield_codes(b"a")?.1, vec!['a']);
        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_comparison() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_comparison(b" [a0]  ==  'abc' ")?.1,
            SubfieldMatcher::Comparison(
                vec!['a', '0'],
                ComparisonOp::Eq,
                BString::from("abc")
            )
        );
        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_regex() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_regex(b"[a0] =~ '^abc'")?.1,
            SubfieldMatcher::Regex(vec!['a', '0'], String::from("^abc"), false)
        );

        assert_eq!(
            parse_subfield_matcher_regex(b"[a0] !~ '^abc'")?.1,
            SubfieldMatcher::Regex(vec!['a', '0'], String::from("^abc"), true)
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_in() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_in(b"[a0] in ['ab', 'cd']")?.1,
            SubfieldMatcher::In(
                vec!['a', '0'],
                vec![BString::from("ab"), BString::from("cd")],
                false
            )
        );

        assert_eq!(
            parse_subfield_matcher_in(b"[a0] not in ['ab', 'cd']")?.1,
            SubfieldMatcher::In(
                vec!['a', '0'],
                vec![BString::from("ab"), BString::from("cd")],
                true
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_exists() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_exists(b" [a0]? ")?.1,
            SubfieldMatcher::Exists(vec!['a', '0'],)
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher() -> TestResult {
        assert!(parse_subfield_matcher(b"[a0] == 'abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] != 'abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] =$ 'abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] =^ 'abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] =~ '^abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] !~ '^abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] in ['a', 'b']").is_ok());
        assert!(parse_subfield_matcher(b"[a0] not in ['a', 'b']").is_ok());
        assert!(parse_subfield_matcher(b"[a0]?").is_ok());
        Ok(())
    }
}
