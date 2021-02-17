//! Filter Expressions

use crate::legacy::{
    parse_field_tag, parse_occurrence_matcher, parse_string,
    parse_subfield_name, Field, OccurrenceMatcher, Record,
};
use crate::utils::ws;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, map, opt, verify};
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{Finish, IResult};

use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum BooleanOp {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Re,
    StartsWith,
    EndsWith,
    In,
}

#[derive(Debug, PartialEq)]
pub enum SubfieldFilter {
    Comparison(char, ComparisonOp, Vec<String>),
    Boolean(Box<SubfieldFilter>, BooleanOp, Box<SubfieldFilter>),
    Grouped(Box<SubfieldFilter>),
    Exists(char),
    Not(Box<SubfieldFilter>),
}

impl SubfieldFilter {
    pub fn matches(&self, field: &Field) -> bool {
        match self {
            SubfieldFilter::Comparison(name, op, values) => match op {
                ComparisonOp::Eq => field.iter().any(|subfield| {
                    subfield.name() == *name && subfield.value() == values[0]
                }),
                ComparisonOp::Ne => field.iter().all(|subfield| {
                    subfield.name() == *name && subfield.value() != values[0]
                }),
                ComparisonOp::StartsWith => field.iter().any(|subfield| {
                    subfield.name() == *name
                        && subfield.value().starts_with(&values[0])
                }),
                ComparisonOp::EndsWith => field.iter().any(|subfield| {
                    subfield.name() == *name
                        && subfield.value().ends_with(&values[0])
                }),
                ComparisonOp::Re => {
                    let re = Regex::new(&values[0]).unwrap();
                    field.iter().any(|subfield| {
                        subfield.name() == *name
                            && re.is_match(subfield.value())
                    })
                }
                ComparisonOp::In => field.iter().any(|subfield| {
                    subfield.name() == *name
                        && values.contains(&String::from(subfield.value()))
                }),
            },
            SubfieldFilter::Boolean(lhs, op, rhs) => match op {
                BooleanOp::And => lhs.matches(field) && rhs.matches(field),
                BooleanOp::Or => lhs.matches(field) || rhs.matches(field),
            },
            SubfieldFilter::Grouped(filter) => filter.matches(field),
            SubfieldFilter::Not(filter) => !filter.matches(field),
            SubfieldFilter::Exists(name) => {
                field.iter().any(|subfield| subfield.name() == *name)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Filter<'a> {
    Field(String, OccurrenceMatcher<'a>, SubfieldFilter),
    Boolean(Box<Filter<'a>>, BooleanOp, Box<Filter<'a>>),
    Exists(String, OccurrenceMatcher<'a>),
    Grouped(Box<Filter<'a>>),
    Not(Box<Filter<'a>>),
}

impl<'a> Filter<'a> {
    pub fn matches(&self, record: &Record) -> bool {
        match self {
            Filter::Field(tag, occurrence, filter) => {
                record.iter().any(|field| {
                    field.tag() == tag
                        && *occurrence == field.occurrence()
                        && filter.matches(field)
                })
            }
            Filter::Exists(tag, occurrence) => record.iter().any(|field| {
                field.tag() == tag && *occurrence == field.occurrence()
            }),
            Filter::Boolean(lhs, op, rhs) => match op {
                BooleanOp::And => lhs.matches(record) && rhs.matches(record),
                BooleanOp::Or => lhs.matches(record) || rhs.matches(record),
            },
            Filter::Grouped(filter) => filter.matches(record),
            Filter::Not(filter) => !filter.matches(record),
        }
    }
}

/// Parses a boolean operator (AND (&&) or OR (||)) operator, if possible.
fn parse_boolean_op(i: &str) -> IResult<&str, BooleanOp> {
    alt((
        map(tag("&&"), |_| BooleanOp::And),
        map(tag("||"), |_| BooleanOp::Or),
    ))(i)
}

/// Parses a comparison operator (Equal (==), Not Equal (!=) or Regex (=~).
fn parse_comparison_op(i: &str) -> IResult<&str, ComparisonOp> {
    alt((
        map(tag("=="), |_| ComparisonOp::Eq),
        map(tag("!="), |_| ComparisonOp::Ne),
        map(tag("=^"), |_| ComparisonOp::StartsWith),
        map(tag("=$"), |_| ComparisonOp::EndsWith),
        map(tag("=~"), |_| ComparisonOp::Re),
    ))(i)
}

fn parse_subfield_regex(i: &str) -> IResult<&str, SubfieldFilter> {
    map(
        tuple((
            ws(parse_subfield_name),
            map(ws(tag("=~")), |_| ComparisonOp::Re),
            verify(ws(parse_string), |s| Regex::new(s).is_ok()),
        )),
        |(name, op, regex)| SubfieldFilter::Comparison(name, op, vec![regex]),
    )(i)
}

/// Parses a subfield comparison expression.
fn parse_subfield_comparison(i: &str) -> IResult<&str, SubfieldFilter> {
    map(
        tuple((
            ws(parse_subfield_name),
            ws(parse_comparison_op),
            ws(parse_string),
        )),
        |(name, op, value)| SubfieldFilter::Comparison(name, op, vec![value]),
    )(i)
}

fn parse_subfield_in_expr(i: &str) -> IResult<&str, SubfieldFilter> {
    map(
        tuple((
            ws(parse_subfield_name),
            map(tag("in"), |_| ComparisonOp::In),
            delimited(
                ws(char('[')),
                separated_list1(ws(char(',')), parse_string),
                ws(char(']')),
            ),
        )),
        |(name, op, values)| SubfieldFilter::Comparison(name, op, values),
    )(i)
}

/// Parses a subfield exists expression.
fn parse_subfield_exists(i: &str) -> IResult<&str, SubfieldFilter> {
    map(terminated(parse_subfield_name, char('?')), |name| {
        SubfieldFilter::Exists(name)
    })(i)
}

/// Parses a subfield group expression.
fn parse_subfield_group(i: &str) -> IResult<&str, SubfieldFilter> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(parse_subfield_filter, char(')'))),
        ),
        |e| SubfieldFilter::Grouped(Box::new(e)),
    )(i)
}

/// Parses a subfield not expression.
fn parse_subfield_not_expr(i: &str) -> IResult<&str, SubfieldFilter> {
    map(
        preceded(
            ws(char('!')),
            cut(alt((parse_subfield_exists, parse_subfield_group))),
        ),
        |e| SubfieldFilter::Not(Box::new(e)),
    )(i)
}

fn parse_subfield_primary(i: &str) -> IResult<&str, SubfieldFilter> {
    alt((
        parse_subfield_comparison,
        parse_subfield_regex,
        parse_subfield_not_expr,
        parse_subfield_in_expr,
        parse_subfield_exists,
        parse_subfield_group,
    ))(i)
}

fn parse_subfield_boolean_expr(i: &str) -> IResult<&str, SubfieldFilter> {
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

fn parse_subfield_filter(i: &str) -> IResult<&str, SubfieldFilter> {
    alt((parse_subfield_boolean_expr, parse_subfield_primary))(i)
}

fn parse_field_complex(i: &str) -> IResult<&str, Filter> {
    map(
        tuple((
            pair(parse_field_tag, opt(parse_occurrence_matcher)),
            preceded(
                ws(char('{')),
                cut(terminated(parse_subfield_filter, ws(char('}')))),
            ),
        )),
        |((tag, occurrence), filter)| {
            Filter::Field(
                String::from(tag),
                occurrence.unwrap_or(OccurrenceMatcher::None),
                filter,
            )
        },
    )(i)
}

fn parse_field_simple(i: &str) -> IResult<&str, Filter> {
    map(
        tuple((
            pair(parse_field_tag, opt(parse_occurrence_matcher)),
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
                String::from(tag),
                occurrence.unwrap_or(OccurrenceMatcher::None),
                filter,
            )
        },
    )(i)
}

fn parse_field_exists(i: &str) -> IResult<&str, Filter> {
    map(
        terminated(
            pair(parse_field_tag, opt(parse_occurrence_matcher)),
            char('?'),
        ),
        |(tag, occurrence)| {
            Filter::Exists(
                String::from(tag),
                occurrence.unwrap_or(OccurrenceMatcher::None),
            )
        },
    )(i)
}

fn parse_field_expr(i: &str) -> IResult<&str, Filter> {
    alt((parse_field_simple, parse_field_complex, parse_field_exists))(i)
}

fn parse_field_group(i: &str) -> IResult<&str, Filter> {
    map(
        preceded(ws(char('(')), cut(terminated(parse_filter_expr, char(')')))),
        |e| Filter::Grouped(Box::new(e)),
    )(i)
}

fn parse_field_not_expr(i: &str) -> IResult<&str, Filter> {
    map(preceded(ws(char('!')), cut(parse_field_primary)), |e| {
        Filter::Not(Box::new(e))
    })(i)
}

fn parse_field_primary(i: &str) -> IResult<&str, Filter> {
    alt((parse_field_group, parse_field_expr, parse_field_not_expr))(i)
}

fn parse_field_boolean_expr(i: &str) -> IResult<&str, Filter> {
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

fn parse_filter_expr(i: &str) -> IResult<&str, Filter> {
    alt((parse_field_boolean_expr, parse_field_primary))(i)
}

fn parse_filter(i: &str) -> IResult<&str, Filter> {
    parse_filter_expr(i)
}

#[derive(Debug, PartialEq)]
pub struct ParseFilterError;

impl<'a> Filter<'a> {
    pub fn decode(s: &'a str) -> Result<Self, ParseFilterError> {
        match all_consuming(parse_filter)(s).finish() {
            Ok((_, filter)) => Ok(filter),
            _ => Err(ParseFilterError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn test_parse_boolean_op() {
        assert_eq!(parse_boolean_op("&&"), Ok(("", BooleanOp::And)));
        assert_eq!(parse_boolean_op("||"), Ok(("", BooleanOp::Or)));
    }

    #[test]
    fn test_parse_comparison_op() {
        assert_eq!(parse_comparison_op("=="), Ok(("", ComparisonOp::Eq)));
        assert_eq!(parse_comparison_op("!="), Ok(("", ComparisonOp::Ne)));
        assert_eq!(
            parse_comparison_op("=^"),
            Ok(("", ComparisonOp::StartsWith))
        );
        assert_eq!(parse_comparison_op("=$"), Ok(("", ComparisonOp::EndsWith)));
        assert_eq!(parse_comparison_op("=~"), Ok(("", ComparisonOp::Re)));
    }

    #[test]
    fn test_parse_subfield_comparison() {
        let filter = SubfieldFilter::Comparison(
            '0',
            ComparisonOp::Eq,
            vec!["123456789X".to_string()],
        );
        assert_eq!(
            parse_subfield_comparison("0 == '123456789X'"),
            Ok(("", filter))
        );
    }

    #[test]
    fn test_parse_subfield_regex() {
        assert_eq!(
            parse_subfield_regex("0 =~ '^Tp[123]$'"),
            Ok((
                "",
                SubfieldFilter::Comparison(
                    '0',
                    ComparisonOp::Re,
                    vec!["^Tp[123]$".to_string()],
                )
            ))
        );

        assert!(parse_subfield_regex("0 =~ '^Tp[123]($'").is_err());
    }

    #[test]
    fn test_parse_subfield_in_op() {
        let filter = SubfieldFilter::Comparison(
            '0',
            ComparisonOp::In,
            vec![
                "123456789X".to_string(),
                "123456789Y".to_string(),
                "123456789Z".to_string(),
            ],
        );
        assert_eq!(
            parse_subfield_in_expr(
                "0 in ['123456789X', '123456789Y', '123456789Z']"
            ),
            Ok(("", filter))
        );
    }

    #[test]
    fn test_parse_subfield_exists() {
        assert_eq!(
            parse_subfield_exists("0?"),
            Ok(("", SubfieldFilter::Exists('0')))
        );
    }

    #[test]
    fn test_parse_subfield_gorup() {
        assert_eq!(
            parse_subfield_group("((0?))"),
            Ok((
                "",
                SubfieldFilter::Grouped(Box::new(SubfieldFilter::Grouped(
                    Box::new(SubfieldFilter::Exists('0'))
                ),))
            ))
        );
    }

    #[test]
    fn test_subfield_not_expr() {
        assert_eq!(
            parse_subfield_not_expr("!(!a?)"),
            Ok((
                "",
                SubfieldFilter::Not(Box::new(SubfieldFilter::Grouped(
                    Box::new(SubfieldFilter::Not(Box::new(
                        SubfieldFilter::Exists('a')
                    )))
                )))
            ))
        );
    }

    #[test]
    fn test_parse_subfield_boolean() {
        assert_eq!(
            parse_subfield_boolean_expr("0? && a?"),
            Ok((
                "",
                SubfieldFilter::Boolean(
                    Box::new(SubfieldFilter::Exists('0')),
                    BooleanOp::And,
                    Box::new(SubfieldFilter::Exists('a'))
                )
            ))
        );

        assert_eq!(
            parse_subfield_boolean_expr("0? || a?"),
            Ok((
                "",
                SubfieldFilter::Boolean(
                    Box::new(SubfieldFilter::Exists('0')),
                    BooleanOp::Or,
                    Box::new(SubfieldFilter::Exists('a'))
                )
            ))
        );
    }

    #[test]
    fn test_parse_field_complex() {
        let field_expr = Filter::Field(
            "012A".to_string(),
            OccurrenceMatcher::Value(Cow::Borrowed("000")),
            SubfieldFilter::Boolean(
                Box::new(SubfieldFilter::Exists('0')),
                BooleanOp::Or,
                Box::new(SubfieldFilter::Comparison(
                    'a',
                    ComparisonOp::Eq,
                    vec!["abc".to_string()],
                )),
            ),
        );

        assert_eq!(
            parse_field_complex("012A/000{0? || a == 'abc'}"),
            Ok(("", field_expr))
        );
    }

    #[test]
    fn test_parse_field_exists() {
        let field_expr = Filter::Exists(
            "012A".to_string(),
            OccurrenceMatcher::Value(Cow::Borrowed("00")),
        );
        assert_eq!(parse_field_exists("012A/00?"), Ok(("", field_expr)));

        let field_expr =
            Filter::Exists("012A".to_string(), OccurrenceMatcher::None);
        assert_eq!(parse_field_exists("012A?"), Ok(("", field_expr)));
    }

    #[test]
    fn test_parse_field_simple() {
        let field_expr = Filter::Field(
            "003@".to_string(),
            OccurrenceMatcher::None,
            SubfieldFilter::Comparison(
                '0',
                ComparisonOp::Eq,
                vec!["abc".to_string()],
            ),
        );

        assert_eq!(parse_field_simple("003@.0 == 'abc'"), Ok(("", field_expr)));
    }

    #[test]
    fn test_parse_field_group() {
        let field_expr = Filter::Grouped(Box::new(Filter::Field(
            "003@".to_string(),
            OccurrenceMatcher::None,
            SubfieldFilter::Comparison(
                '0',
                ComparisonOp::Eq,
                vec!["abc".to_string()],
            ),
        )));

        assert_eq!(
            parse_field_group("(003@.0 == 'abc')"),
            Ok(("", field_expr))
        );
    }

    #[test]
    fn test_parse_field_boolean_expr() {
        let filter_expr = Filter::Boolean(
            Box::new(Filter::Field(
                "003@".to_string(),
                OccurrenceMatcher::None,
                SubfieldFilter::Comparison(
                    '0',
                    ComparisonOp::Eq,
                    vec!["abc".to_string()],
                ),
            )),
            BooleanOp::And,
            Box::new(Filter::Field(
                "012A".to_string(),
                OccurrenceMatcher::None,
                SubfieldFilter::Boolean(
                    Box::new(SubfieldFilter::Exists('a')),
                    BooleanOp::And,
                    Box::new(SubfieldFilter::Exists('b')),
                ),
            )),
        );

        assert_eq!(
            parse_field_boolean_expr("003@.0 == 'abc' && 012A{a? && b?}"),
            Ok(("", filter_expr))
        );
    }

    #[test]
    fn test_decode() {
        let expected = Filter::Field(
            "003@".to_string(),
            OccurrenceMatcher::None,
            SubfieldFilter::Comparison(
                '0',
                ComparisonOp::Eq,
                vec!["123456789X".to_string()],
            ),
        );

        assert_eq!(Filter::decode("003@.0 == '123456789X'").unwrap(), expected);
    }
}
