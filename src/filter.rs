//! Filter Expressions

use bstr::{BString, ByteSlice};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, multispace1, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, value, verify};
use nom::error::{FromExternalError, ParseError};
use nom::multi::{fold_many0, many0, many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{Finish, IResult};
use regex::{Regex, RegexBuilder};
use std::cmp::PartialEq;
use std::str;

use crate::common::{parse_boolean_op, BooleanOp, ParseResult};
use crate::occurrence::{parse_occurrence_matcher, OccurrenceMatcher};
use crate::tag::{parse_tag_matcher, TagMatcher};
use crate::{ByteRecord, Field, Subfield};

#[derive(Debug, PartialEq)]
pub enum ComparisonOp {
    Eq,
    StrictEq,
    Ne,
    Re,
    StartsWith,
    EndsWith,
    In,
}

#[derive(Debug, PartialEq)]
pub enum SubfieldFilter {
    Comparison(Vec<char>, ComparisonOp, Vec<BString>),
    Boolean(Box<SubfieldFilter>, BooleanOp, Box<SubfieldFilter>),
    Grouped(Box<SubfieldFilter>),
    Exists(Vec<char>),
    Not(Box<SubfieldFilter>),
}

impl SubfieldFilter {
    pub fn matches(&self, field: &Field, ignore_case: bool) -> bool {
        let cmp_fn = |x: &BString, y: &BString| -> bool {
            if ignore_case {
                x.to_lowercase() == y.to_lowercase()
            } else {
                x == y
            }
        };
        match self {
            SubfieldFilter::Comparison(codes, op, values) => match op {
                ComparisonOp::Eq => field.iter().any(|subfield| {
                    codes.contains(&subfield.code)
                        && cmp_fn(subfield.value(), &values[0])
                }),
                ComparisonOp::StrictEq => {
                    let subfields = field
                        .iter()
                        .filter(|subfield| codes.contains(&subfield.code))
                        .collect::<Vec<&Subfield>>();

                    !subfields.is_empty()
                        && subfields.iter().all(|subfield| {
                            cmp_fn(subfield.value(), &values[0])
                        })
                }
                ComparisonOp::Ne => {
                    let subfields = field
                        .iter()
                        .filter(|subfield| codes.contains(&subfield.code))
                        .collect::<Vec<&Subfield>>();

                    subfields.is_empty()
                        || subfields.iter().all(|subfield| {
                            !cmp_fn(subfield.value(), &values[0])
                        })
                }
                ComparisonOp::StartsWith => field.iter().any(|subfield| {
                    codes.contains(&subfield.code)
                        && if ignore_case {
                            subfield
                                .value
                                .to_ascii_lowercase()
                                .starts_with(&values[0].to_lowercase())
                        } else {
                            subfield.value.starts_with(&values[0])
                        }
                }),
                ComparisonOp::EndsWith => field.iter().any(|subfield| {
                    codes.contains(&subfield.code)
                        && if ignore_case {
                            subfield
                                .value
                                .to_ascii_lowercase()
                                .ends_with(&values[0].to_lowercase())
                        } else {
                            subfield.value.ends_with(&values[0])
                        }
                }),
                ComparisonOp::Re => {
                    // SAFETY: It's safe to call `unwrap()` because the parser
                    // verified that the regular expression is `ok`.
                    let re = RegexBuilder::new(unsafe {
                        str::from_utf8_unchecked(values[0].as_bytes())
                    })
                    .case_insensitive(ignore_case)
                    .build()
                    .unwrap();

                    field.iter().any(|subfield| {
                        let value =
                            String::from_utf8(subfield.value.to_vec()).unwrap();
                        codes.contains(&subfield.code) && re.is_match(&value)
                    })
                }
                ComparisonOp::In => field.iter().any(|subfield| {
                    codes.contains(&subfield.code)
                        && values
                            .iter()
                            .any(|x: &BString| cmp_fn(x, subfield.value()))
                }),
            },
            SubfieldFilter::Boolean(lhs, op, rhs) => match op {
                BooleanOp::And => {
                    lhs.matches(field, ignore_case)
                        && rhs.matches(field, ignore_case)
                }
                BooleanOp::Or => {
                    lhs.matches(field, ignore_case)
                        || rhs.matches(field, ignore_case)
                }
            },
            SubfieldFilter::Grouped(filter) => {
                filter.matches(field, ignore_case)
            }
            SubfieldFilter::Not(filter) => !filter.matches(field, ignore_case),
            SubfieldFilter::Exists(codes) => {
                field.iter().any(|subfield| codes.contains(&subfield.code))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Filter {
    Field(TagMatcher, OccurrenceMatcher, SubfieldFilter),
    Boolean(Box<Filter>, BooleanOp, Box<Filter>),
    Exists(TagMatcher, OccurrenceMatcher),
    Grouped(Box<Filter>),
    Not(Box<Filter>),
    True,
}

impl<'a> Filter {
    pub fn matches(&self, record: &ByteRecord, ignore_case: bool) -> bool {
        match self {
            Filter::Field(tag, occurrence, filter) => {
                record.iter().any(|field| {
                    &field.tag == tag
                        && field.occurrence == *occurrence
                        && filter.matches(field, ignore_case)
                })
            }
            Filter::Exists(tag, occurrence) => record.iter().any(|field| {
                &field.tag == tag && field.occurrence == *occurrence
            }),
            Filter::Boolean(lhs, op, rhs) => match op {
                BooleanOp::And => {
                    lhs.matches(record, ignore_case)
                        && rhs.matches(record, ignore_case)
                }
                BooleanOp::Or => {
                    lhs.matches(record, ignore_case)
                        || rhs.matches(record, ignore_case)
                }
            },
            Filter::Grouped(filter) => filter.matches(record, ignore_case),
            Filter::Not(filter) => !filter.matches(record, ignore_case),
            Filter::True => true,
        }
    }
}

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<'a, F: 'a, O, E: ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char<'a, E>(i: &'a [u8]) -> IResult<&'a [u8], char, E>
where
    E: ParseError<&'a [u8]>
        + FromExternalError<&'a [u8], std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        alt((
            // parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(i)
}

/// Parse a non-empty block of text that doesn't include \ or ".
fn parse_literal<'a, E: ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], &'a [u8], E> {
    verify(is_not("\'\\"), |s: &[u8]| !s.is_empty())(i)
}

#[derive(Debug, Clone)]
enum StringFragment<'a> {
    Literal(&'a [u8]),
    EscapedChar(char),
    EscapedWs,
}

/// Combine parse_literal, parse_escaped_char into a StringFragment.
fn parse_fragment<'a, E>(
    i: &'a [u8],
) -> IResult<&'a [u8], StringFragment<'a>, E>
where
    E: ParseError<&'a [u8]>
        + FromExternalError<&'a [u8], std::num::ParseIntError>,
{
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWs, preceded(char('\\'), multispace1)),
    ))(i)
}

pub(crate) fn parse_string<'a, E>(i: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>
        + FromExternalError<&'a [u8], std::num::ParseIntError>,
{
    map(
        delimited(
            char('\''),
            fold_many0(parse_fragment, Vec::new, |mut string, fragment| {
                match fragment {
                    StringFragment::Literal(s) => string.extend_from_slice(s),
                    StringFragment::EscapedChar(c) => string.push(c as u8),
                    StringFragment::EscapedWs => {}
                }
                string
            }),
            char('\''),
        ),
        // FIXME
        |x| String::from_utf8(x).unwrap(),
    )(i)
}

/// Parses a subfield code.
pub(crate) fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

/// Parses multiple subfield codes.
pub(crate) fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        map(parse_subfield_code, |x| vec![x]),
        delimited(ws(char('[')), many1(ws(parse_subfield_code)), ws(char(']'))),
    ))(i)
}

/// Parses a comparison operator.
fn parse_comparison_op(i: &[u8]) -> ParseResult<ComparisonOp> {
    alt((
        map(tag("==="), |_| ComparisonOp::StrictEq),
        map(tag("=="), |_| ComparisonOp::Eq),
        map(tag("!="), |_| ComparisonOp::Ne),
        map(tag("=^"), |_| ComparisonOp::StartsWith),
        map(tag("=$"), |_| ComparisonOp::EndsWith),
        map(tag("=~"), |_| ComparisonOp::Re),
    ))(i)
}

fn parse_subfield_regex(i: &[u8]) -> ParseResult<SubfieldFilter> {
    map(
        tuple((
            ws(parse_subfield_codes),
            map(ws(tag("=~")), |_| ComparisonOp::Re),
            verify(ws(parse_string), |s| Regex::new(s).is_ok()),
        )),
        |(names, op, regex)| {
            SubfieldFilter::Comparison(names, op, vec![BString::from(regex)])
        },
    )(i)
}

/// Parses a subfield comparison expression.
fn parse_subfield_comparison(i: &[u8]) -> ParseResult<SubfieldFilter> {
    map(
        tuple((
            ws(parse_subfield_codes),
            ws(parse_comparison_op),
            ws(parse_string),
        )),
        |(names, op, value)| {
            SubfieldFilter::Comparison(names, op, vec![BString::from(value)])
        },
    )(i)
}

fn parse_subfield_in_expr(i: &[u8]) -> ParseResult<SubfieldFilter> {
    map(
        tuple((
            ws(parse_subfield_codes),
            opt(ws(tag("not"))),
            map(tag("in"), |_| ComparisonOp::In),
            delimited(
                ws(char('[')),
                separated_list1(ws(char(',')), parse_string),
                ws(char(']')),
            ),
        )),
        |(names, negate, op, values)| {
            let filter = SubfieldFilter::Comparison(
                names,
                op,
                values.iter().map(|x| BString::from(x.as_bytes())).collect(),
            );
            if negate.is_some() {
                SubfieldFilter::Not(Box::new(filter))
            } else {
                filter
            }
        },
    )(i)
}

/// Parses a subfield exists expression.
fn parse_subfield_exists(i: &[u8]) -> ParseResult<SubfieldFilter> {
    map(terminated(parse_subfield_codes, char('?')), |names| {
        SubfieldFilter::Exists(names)
    })(i)
}

/// Parses a subfield group expression.
fn parse_subfield_group(i: &[u8]) -> ParseResult<SubfieldFilter> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(parse_subfield_filter, char(')'))),
        ),
        |e| SubfieldFilter::Grouped(Box::new(e)),
    )(i)
}

/// Parses a subfield not expression.
fn parse_subfield_not_expr(i: &[u8]) -> ParseResult<SubfieldFilter> {
    map(
        preceded(
            ws(char('!')),
            cut(alt((parse_subfield_exists, parse_subfield_group))),
        ),
        |e| SubfieldFilter::Not(Box::new(e)),
    )(i)
}

fn parse_subfield_primary(i: &[u8]) -> ParseResult<SubfieldFilter> {
    alt((
        parse_subfield_comparison,
        parse_subfield_regex,
        parse_subfield_not_expr,
        parse_subfield_in_expr,
        parse_subfield_exists,
        parse_subfield_group,
    ))(i)
}

fn parse_subfield_boolean_expr(i: &[u8]) -> ParseResult<SubfieldFilter> {
    let (i, (first, remainder)) = tuple((
        parse_subfield_primary,
        many0(pair(ws(parse_boolean_op), ws(parse_subfield_primary))),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, (op, next)| {
            SubfieldFilter::Boolean(Box::new(prev), op, Box::new(next))
        }),
    ))
}

pub(crate) fn parse_subfield_filter(i: &[u8]) -> ParseResult<SubfieldFilter> {
    alt((parse_subfield_boolean_expr, parse_subfield_primary))(i)
}

fn parse_field_complex(i: &[u8]) -> ParseResult<Filter> {
    map(
        tuple((
            pair(parse_tag_matcher, opt(parse_occurrence_matcher)),
            preceded(
                ws(char('{')),
                cut(terminated(parse_subfield_filter, ws(char('}')))),
            ),
        )),
        |((tag, occurrence), filter)| {
            Filter::Field(
                tag,
                occurrence.unwrap_or(OccurrenceMatcher::None),
                filter,
            )
        },
    )(i)
}

fn parse_field_simple(i: &[u8]) -> ParseResult<Filter> {
    map(
        tuple((
            pair(parse_tag_matcher, opt(parse_occurrence_matcher)),
            preceded(
                ws(char('.')),
                cut(alt((
                    parse_subfield_comparison,
                    parse_subfield_exists,
                    parse_subfield_in_expr,
                ))),
            ),
        )),
        |((tag, occurrence), filter)| {
            Filter::Field(
                tag,
                occurrence.unwrap_or(OccurrenceMatcher::None),
                filter,
            )
        },
    )(i)
}

fn parse_field_exists(i: &[u8]) -> ParseResult<Filter> {
    map(
        terminated(
            pair(parse_tag_matcher, opt(parse_occurrence_matcher)),
            char('?'),
        ),
        |(tag, occurrence)| {
            Filter::Exists(tag, occurrence.unwrap_or(OccurrenceMatcher::None))
        },
    )(i)
}

fn parse_field_expr(i: &[u8]) -> ParseResult<Filter> {
    alt((parse_field_simple, parse_field_complex, parse_field_exists))(i)
}

fn parse_field_group(i: &[u8]) -> ParseResult<Filter> {
    map(
        preceded(ws(char('(')), cut(terminated(parse_filter_expr, char(')')))),
        |e| Filter::Grouped(Box::new(e)),
    )(i)
}

fn parse_field_not_expr(i: &[u8]) -> ParseResult<Filter> {
    map(preceded(ws(char('!')), cut(parse_field_primary)), |e| {
        Filter::Not(Box::new(e))
    })(i)
}

fn parse_field_primary(i: &[u8]) -> ParseResult<Filter> {
    alt((parse_field_group, parse_field_expr, parse_field_not_expr))(i)
}

fn parse_field_boolean_expr(i: &[u8]) -> ParseResult<Filter> {
    let (i, (first, remainder)) = tuple((
        parse_field_primary,
        many0(pair(ws(parse_boolean_op), ws(parse_field_primary))),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, (op, next)| {
            Filter::Boolean(Box::new(prev), op, Box::new(next))
        }),
    ))
}

fn parse_filter_expr(i: &[u8]) -> ParseResult<Filter> {
    alt((parse_field_boolean_expr, parse_field_primary))(i)
}

fn parse_filter(i: &[u8]) -> ParseResult<Filter> {
    ws(parse_filter_expr)(i)
}

#[derive(Debug, PartialEq)]
pub struct ParseFilterError;

impl Filter {
    pub fn decode(s: &str) -> std::result::Result<Self, ParseFilterError> {
        match all_consuming(parse_filter)(s.as_bytes()).finish() {
            Ok((_, filter)) => Ok(filter),
            _ => Err(ParseFilterError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::TestResult;

    #[test]
    fn test_parse_boolean_op() -> TestResult {
        assert_eq!(parse_boolean_op(b"&&")?.1, BooleanOp::And);
        assert_eq!(parse_boolean_op(b"||")?.1, BooleanOp::Or);

        Ok(())
    }

    #[test]
    fn test_parse_comparison_op() -> TestResult {
        assert_eq!(parse_comparison_op(b"==")?.1, ComparisonOp::Eq);
        assert_eq!(parse_comparison_op(b"!=")?.1, ComparisonOp::Ne);
        assert_eq!(parse_comparison_op(b"=^")?.1, ComparisonOp::StartsWith);
        assert_eq!(parse_comparison_op(b"=$")?.1, ComparisonOp::EndsWith);
        assert_eq!(parse_comparison_op(b"=~")?.1, ComparisonOp::Re);

        Ok(())
    }

    #[test]
    fn test_parse_subfield_comparison() -> TestResult {
        let filter = SubfieldFilter::Comparison(
            vec!['0'],
            ComparisonOp::Eq,
            vec![BString::from("123456789X")],
        );
        assert_eq!(parse_subfield_comparison(b"0 == '123456789X'")?.1, filter);

        Ok(())
    }

    #[test]
    fn test_parse_subfield_regex() -> TestResult {
        assert_eq!(
            parse_subfield_regex(b"0 =~ '^Tp[123]$'")?.1,
            SubfieldFilter::Comparison(
                vec!['0'],
                ComparisonOp::Re,
                vec![BString::from("^Tp[123]$")]
            )
        );

        assert!(parse_subfield_regex(b"0 =~ '^Tp[123]($'").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_subfield_in_op() -> TestResult {
        let filter = SubfieldFilter::Comparison(
            vec!['0'],
            ComparisonOp::In,
            vec![
                BString::from("123456789X"),
                BString::from("123456789Y"),
                BString::from("123456789Z"),
            ],
        );
        assert_eq!(
            parse_subfield_in_expr(
                b"0 in ['123456789X', '123456789Y', '123456789Z']"
            )?
            .1,
            filter
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_exists() -> TestResult {
        assert_eq!(
            parse_subfield_exists(b"0?")?.1,
            SubfieldFilter::Exists(vec!['0'])
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_gorup() -> TestResult {
        assert_eq!(
            parse_subfield_group(b"((0?))")?.1,
            SubfieldFilter::Grouped(Box::new(SubfieldFilter::Grouped(
                Box::new(SubfieldFilter::Exists(vec!['0']))
            )))
        );

        Ok(())
    }

    #[test]
    fn test_subfield_not_expr() -> TestResult {
        assert_eq!(
            parse_subfield_not_expr(b"!(!a?)")?.1,
            SubfieldFilter::Not(Box::new(SubfieldFilter::Grouped(Box::new(
                SubfieldFilter::Not(Box::new(SubfieldFilter::Exists(vec![
                    'a'
                ])))
            ))))
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_boolean() -> TestResult {
        assert_eq!(
            parse_subfield_boolean_expr(b"0? && a?")?.1,
            SubfieldFilter::Boolean(
                Box::new(SubfieldFilter::Exists(vec!['0'])),
                BooleanOp::And,
                Box::new(SubfieldFilter::Exists(vec!['a']))
            )
        );

        assert_eq!(
            parse_subfield_boolean_expr(b"0? || a?")?.1,
            SubfieldFilter::Boolean(
                Box::new(SubfieldFilter::Exists(vec!['0'])),
                BooleanOp::Or,
                Box::new(SubfieldFilter::Exists(vec!['a']))
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_field_complex() -> TestResult {
        let field_expr = Filter::Field(
            TagMatcher::new("012A")?,
            OccurrenceMatcher::None,
            SubfieldFilter::Boolean(
                Box::new(SubfieldFilter::Exists(vec!['0'])),
                BooleanOp::Or,
                Box::new(SubfieldFilter::Comparison(
                    vec!['a'],
                    ComparisonOp::Eq,
                    vec![BString::from("abc")],
                )),
            ),
        );

        assert_eq!(
            parse_field_complex(b"012A/00{0? || a == 'abc'}")?.1,
            field_expr
        );

        Ok(())
    }

    #[test]
    fn test_parse_field_exists() -> TestResult {
        let field_expr = Filter::Exists(
            TagMatcher::new("012A")?,
            OccurrenceMatcher::new("01").unwrap(),
        );
        assert_eq!(parse_field_exists(b"012A/01?")?.1, field_expr);

        let field_expr =
            Filter::Exists(TagMatcher::new("012A")?, OccurrenceMatcher::None);
        assert_eq!(parse_field_exists(b"012A?")?.1, field_expr);

        Ok(())
    }

    #[test]
    fn test_parse_field_simple() -> TestResult {
        let field_expr = Filter::Field(
            TagMatcher::new("003@")?,
            OccurrenceMatcher::None,
            SubfieldFilter::Comparison(
                vec!['0'],
                ComparisonOp::Eq,
                vec![BString::from("abc")],
            ),
        );

        assert_eq!(parse_field_simple(b"003@.0 == 'abc'")?.1, field_expr);

        Ok(())
    }

    #[test]
    fn test_parse_not_expr() -> TestResult {
        let field_expr = Filter::Not(Box::new(Filter::Exists(
            TagMatcher::new("003@")?,
            OccurrenceMatcher::None,
        )));

        assert_eq!(parse_field_not_expr(b"!003@?")?.1, field_expr);

        Ok(())
    }

    #[test]
    fn test_parse_field_group() -> TestResult {
        let field_expr = Filter::Grouped(Box::new(Filter::Field(
            TagMatcher::new("003@")?,
            OccurrenceMatcher::None,
            SubfieldFilter::Comparison(
                vec!['0'],
                ComparisonOp::Eq,
                vec![BString::from("abc")],
            ),
        )));

        assert_eq!(parse_field_group(b"(003@.0 == 'abc')")?.1, field_expr);

        Ok(())
    }

    #[test]
    fn test_parse_field_boolean_expr() -> TestResult {
        let filter_expr = Filter::Boolean(
            Box::new(Filter::Field(
                TagMatcher::new("003@")?,
                OccurrenceMatcher::None,
                SubfieldFilter::Comparison(
                    vec!['0'],
                    ComparisonOp::Eq,
                    vec![BString::from("abc")],
                ),
            )),
            BooleanOp::And,
            Box::new(Filter::Field(
                TagMatcher::new("012A")?,
                OccurrenceMatcher::None,
                SubfieldFilter::Boolean(
                    Box::new(SubfieldFilter::Exists(vec!['a'])),
                    BooleanOp::And,
                    Box::new(SubfieldFilter::Exists(vec!['b'])),
                ),
            )),
        );

        assert_eq!(
            parse_field_boolean_expr(b"003@.0 == 'abc' && 012A{a? && b?}")?.1,
            filter_expr
        );

        Ok(())
    }

    #[test]
    fn test_decode() -> TestResult {
        let expected = Filter::Field(
            TagMatcher::new("003@")?,
            OccurrenceMatcher::None,
            SubfieldFilter::Comparison(
                vec!['0'],
                ComparisonOp::Eq,
                vec![BString::from("123456789X")],
            ),
        );

        assert_eq!(Filter::decode("003@.0 == '123456789X'").unwrap(), expected);

        Ok(())
    }
}

// #[inline]
// fn parse_subfield_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
//     let (i, (first, remainder)) = tuple((
//         parse_subfield_matcher_primary,
//         many0(pair(
//             ws(parse_boolean_op),
//             ws(parse_subfield_matcher_primary),
//         )),
//     ))(i)?;

//     Ok((
//         i,
//         remainder.into_iter().fold(first, |prev, (op, next)| {
//             SubfieldMatcher::Composite(Box::new(prev), op, Box::new(next))
//         }),
//     ))
// }
