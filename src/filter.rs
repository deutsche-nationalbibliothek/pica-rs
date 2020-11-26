use crate::field::{parse_field_occurrence, parse_field_tag};
use crate::subfield::parse_subfield_code;
use crate::utils::{parse_string, ws};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{cut, map, opt};
use nom::multi::many0;
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::{Finish, IResult};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseFilterError;

#[derive(Debug, PartialEq)]
pub enum ComparisonOp {
    Eq,
    Ne,
}

#[derive(Debug, PartialEq)]
pub enum BooleanOp {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum SubfieldFilter {
    ComparisonExpr(char, ComparisonOp, String),
    BooleanExpr(Box<SubfieldFilter>, BooleanOp, Box<SubfieldFilter>),
    GroupedExpr(Box<SubfieldFilter>),
    ExistsExpr(char),
}

#[derive(Debug, PartialEq)]
pub enum Filter {
    FieldExpr(String, Option<String>, SubfieldFilter),
    BooleanExpr(Box<Filter>, BooleanOp, Box<Filter>),
    GroupedExpr(Box<Filter>),
}

fn parse_comparison_op(i: &str) -> IResult<&str, ComparisonOp> {
    alt((
        map(tag("=="), |_| ComparisonOp::Eq),
        map(tag("!="), |_| ComparisonOp::Ne),
    ))(i)
}

fn parse_boolean_op(i: &str) -> IResult<&str, BooleanOp> {
    alt((
        map(tag("&&"), |_| BooleanOp::And),
        map(tag("||"), |_| BooleanOp::Or),
    ))(i)
}

fn parse_subfield_comparison(i: &str) -> IResult<&str, SubfieldFilter> {
    map(
        tuple((
            ws(parse_subfield_code),
            ws(parse_comparison_op),
            ws(parse_string),
        )),
        |(code, op, value)| SubfieldFilter::ComparisonExpr(code, op, value),
    )(i)
}

fn parse_subfield_exists(i: &str) -> IResult<&str, SubfieldFilter> {
    map(terminated(parse_subfield_code, char('?')), |code| {
        SubfieldFilter::ExistsExpr(code)
    })(i)
}

fn parse_subfield_group(i: &str) -> IResult<&str, SubfieldFilter> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(parse_subfield_filter, char(')'))),
        ),
        |e| SubfieldFilter::GroupedExpr(Box::new(e)),
    )(i)
}

fn parse_subfield_primary(i: &str) -> IResult<&str, SubfieldFilter> {
    alt((
        parse_subfield_comparison,
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
            SubfieldFilter::BooleanExpr(Box::new(prev), op, Box::new(next))
        }),
    ))
}

fn parse_subfield_filter(i: &str) -> IResult<&str, SubfieldFilter> {
    alt((parse_subfield_boolean_expr, parse_subfield_primary))(i)
}

fn parse_field_expr(i: &str) -> IResult<&str, Filter> {
    map(
        tuple((
            pair(parse_field_tag, opt(parse_field_occurrence)),
            preceded(
                ws(char('{')),
                cut(terminated(parse_subfield_filter, ws(char('}')))),
            ),
        )),
        |((tag, occurrence), filter)| {
            Filter::FieldExpr(
                String::from(tag),
                occurrence.map(String::from),
                filter,
            )
        },
    )(i)
}

fn parse_field_group(i: &str) -> IResult<&str, Filter> {
    map(
        preceded(ws(char('(')), cut(terminated(parse_filter_expr, char(')')))),
        |e| Filter::GroupedExpr(Box::new(e)),
    )(i)
}

fn parse_field_primary(i: &str) -> IResult<&str, Filter> {
    alt((parse_field_group, parse_field_expr))(i)
}

fn parse_field_boolean_expr(i: &str) -> IResult<&str, Filter> {
    let (i, (first, remainder)) = tuple((
        parse_field_primary,
        many0(pair(ws(parse_boolean_op), ws(parse_field_primary))),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, (op, next)| {
            Filter::BooleanExpr(Box::new(prev), op, Box::new(next))
        }),
    ))
}

fn parse_filter_expr(i: &str) -> IResult<&str, Filter> {
    alt((parse_field_boolean_expr, parse_field_primary))(i)
}

impl FromStr for Filter {
    type Err = ParseFilterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_filter_expr(s).finish() {
            Ok((_, filter)) => Ok(filter),
            _ => Err(ParseFilterError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filter_expr() {
        let fe1 = Filter::FieldExpr(
            "002@".to_string(),
            None,
            SubfieldFilter::BooleanExpr(
                Box::new(SubfieldFilter::ComparisonExpr(
                    '0',
                    ComparisonOp::Eq,
                    "Tp1".to_string(),
                )),
                BooleanOp::Or,
                Box::new(SubfieldFilter::ComparisonExpr(
                    '0',
                    ComparisonOp::Eq,
                    "Tp2".to_string(),
                )),
            ),
        );

        let fe2 = Filter::FieldExpr(
            "012A".to_string(),
            Some("00".to_string()),
            SubfieldFilter::ComparisonExpr(
                'x',
                ComparisonOp::Eq,
                "abc".to_string(),
            ),
        );

        let fe3 = Filter::FieldExpr(
            "003@".to_string(),
            None,
            SubfieldFilter::ComparisonExpr(
                '0',
                ComparisonOp::Eq,
                "123".to_string(),
            ),
        );

        let filter =
            "(002@{0 == 'Tp1' || 0 == 'Tp2'} || 012A/00{x == 'abc'}) && \
                      003@{0 == '123'}";
        let expected = Filter::BooleanExpr(
            Box::new(Filter::GroupedExpr(Box::new(Filter::BooleanExpr(
                Box::new(fe1),
                BooleanOp::Or,
                Box::new(fe2),
            )))),
            BooleanOp::And,
            Box::new(fe3),
        );
        assert_eq!(parse_filter_expr(filter), Ok(("", expected)));
    }

    #[test]
    fn test_parse_field_boolean_expr() {
        assert_eq!(
            parse_field_boolean_expr("003@{0 == '123'} && 002@{0 == 'Tp1'}"),
            Ok((
                "",
                Filter::BooleanExpr(
                    Box::new(Filter::FieldExpr(
                        "003@".to_string(),
                        None,
                        SubfieldFilter::ComparisonExpr(
                            '0',
                            ComparisonOp::Eq,
                            "123".to_string()
                        )
                    )),
                    BooleanOp::And,
                    Box::new(Filter::FieldExpr(
                        "002@".to_string(),
                        None,
                        SubfieldFilter::ComparisonExpr(
                            '0',
                            ComparisonOp::Eq,
                            "Tp1".to_string()
                        )
                    ))
                )
            ))
        );
    }

    #[test]
    fn test_parse_field_group() {
        assert_eq!(
            parse_field_group("(003@{a == '123' || b == '456'})"),
            Ok((
                "",
                Filter::GroupedExpr(Box::new(Filter::FieldExpr(
                    "003@".to_string(),
                    None,
                    SubfieldFilter::BooleanExpr(
                        Box::new(SubfieldFilter::ComparisonExpr(
                            'a',
                            ComparisonOp::Eq,
                            "123".to_string()
                        )),
                        BooleanOp::Or,
                        Box::new(SubfieldFilter::ComparisonExpr(
                            'b',
                            ComparisonOp::Eq,
                            "456".to_string()
                        )),
                    )
                )))
            ))
        );
    }

    #[test]
    fn test_parse_field_expr() {
        assert_eq!(
            parse_field_expr("012A/00{0 == '123456789X'}"),
            Ok((
                "",
                Filter::FieldExpr(
                    "012A".to_string(),
                    Some("00".to_string()),
                    SubfieldFilter::ComparisonExpr(
                        '0',
                        ComparisonOp::Eq,
                        "123456789X".to_string()
                    )
                )
            ))
        );
    }

    #[test]
    fn test_parse_subfield_filter() {
        assert_eq!(
            parse_subfield_filter("(0? || 1?) && a == 'b'"),
            Ok((
                "",
                SubfieldFilter::BooleanExpr(
                    Box::new(SubfieldFilter::GroupedExpr(Box::new(
                        SubfieldFilter::BooleanExpr(
                            Box::new(SubfieldFilter::ExistsExpr('0')),
                            BooleanOp::Or,
                            Box::new(SubfieldFilter::ExistsExpr('1'))
                        )
                    ))),
                    BooleanOp::And,
                    Box::new(SubfieldFilter::ComparisonExpr(
                        'a',
                        ComparisonOp::Eq,
                        "b".to_string()
                    ))
                )
            ))
        );
    }

    #[test]
    fn test_parse_subfield_boolean_expr() {
        assert_eq!(
            parse_subfield_boolean_expr("0? || a?"),
            Ok((
                "",
                SubfieldFilter::BooleanExpr(
                    Box::new(SubfieldFilter::ExistsExpr('0')),
                    BooleanOp::Or,
                    Box::new(SubfieldFilter::ExistsExpr('a')),
                )
            ))
        );

        assert_eq!(
            parse_subfield_boolean_expr("0? && a?"),
            Ok((
                "",
                SubfieldFilter::BooleanExpr(
                    Box::new(SubfieldFilter::ExistsExpr('0')),
                    BooleanOp::And,
                    Box::new(SubfieldFilter::ExistsExpr('a')),
                )
            ))
        );
    }

    #[test]
    fn test_parse_subfield_group() {
        assert_eq!(
            parse_subfield_group("(0?)"),
            Ok((
                "",
                SubfieldFilter::GroupedExpr(Box::new(
                    SubfieldFilter::ExistsExpr('0')
                ))
            ))
        );
    }

    #[test]
    fn test_parse_subfield_comparison() {
        assert_eq!(
            parse_subfield_comparison("0 == '123456789X'"),
            Ok((
                "",
                SubfieldFilter::ComparisonExpr(
                    '0',
                    ComparisonOp::Eq,
                    "123456789X".to_string()
                )
            ))
        )
    }

    #[test]
    fn test_parse_subfield_exists() {
        assert_eq!(
            parse_subfield_exists("0?"),
            Ok(("", SubfieldFilter::ExistsExpr('0')))
        );
    }

    #[test]
    fn test_parse_boolean_op() {
        assert_eq!(parse_boolean_op("&&"), Ok(("", BooleanOp::And)));
        assert_eq!(parse_boolean_op("||"), Ok(("", BooleanOp::Or)));
    }

    #[test]
    fn test_parse_comparison_op() {
        assert_eq!(parse_comparison_op("=="), Ok(("", ComparisonOp::Eq)));
        assert_eq!(parse_comparison_op("!="), Ok(("", ComparisonOp::Ne)));
    }
}

// #[derive(Debug, PartialEq)]
// pub enum BooleanOp {
//     And,
//     Or,
// }

// #[derive(Debug, PartialEq)]
// pub enum Filter {
// }

// use crate::error::ParsePicaError;
// use crate::parser::parse_filter;
// use crate::Path;
// use std::str::FromStr;

// #[derive(Debug, PartialEq)]
// pub enum ComparisonOp {
//     Eq,
//     Ne,
//     Re,
//     StartsWith,
//     EndsWith,
// }

// #[derive(Debug, PartialEq)]
// pub enum BooleanOp {
//     And,
//     Or,
// }

// #[derive(Debug, PartialEq)]
// pub enum Filter {
//     ComparisonExpr(Path, ComparisonOp, String),
//     ExistenceExpr(Path),
//     BooleanExpr(Box<Filter>, BooleanOp, Box<Filter>),
//     GroupedExpr(Box<Filter>),
//     NotExpr(Box<Filter>),
// }

// impl FromStr for Filter {
//     type Err = ParsePicaError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match parse_filter(s) {
//             Ok((_, path)) => Ok(path),
//             _ => Err(ParsePicaError::InvalidFilter),
//         }
//     }
// }
