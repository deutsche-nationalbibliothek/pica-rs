//! This module provides functions to parse bibliographic records encoded in
//! PICA+ and parsers used in the cli commands. PICA+ is the internal format
//! used by the OCLC library system.
//!
//! NOTE: The code to parse excaped strings is based on the nom example; see
//! https://git.io/JkoOn.
//!
//! # PICA+ Grammar
//!
//! ```text
//! Record     ::= Field*
//! Field      ::= Tag Occurrence? Subfield* #x1e
//! Tag        ::= [012] [0-9]{2} [A-Z@]
//! Occurrence ::= '/' [0-9]{2,3}
//! Subfield   ::= Code Value
//! Code       ::= [a-zA-Z0-9]
//! Value      ::= [^#x1e#x1f]
//! ```
//!
//! [EBNF]: https://www.w3.org/TR/REC-xml/#sec-notation

use crate::filter::{BooleanOp, ComparisonOp};
use crate::{Field, Filter, Path, Record};

use crate::subfield::{parse_subfield, parse_subfield_code};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::streaming::{is_not, take_while_m_n};
use nom::character::complete::{
    char, digit1, multispace0, multispace1, one_of,
};
use nom::combinator::{
    all_consuming, map, map_opt, map_res, opt, recognize, value, verify,
};
use nom::error::{FromExternalError, ParseError};
use nom::multi::{count, fold_many0, many0, many1, many_m_n, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

pub(crate) fn parse_comparison_expr(i: &str) -> IResult<&str, Filter> {
    map(
        tuple((
            ws(parse_path),
            alt((
                map(ws(tag("==")), |_| ComparisonOp::Eq),
                map(ws(tag("!=")), |_| ComparisonOp::Ne),
                map(ws(tag("=~")), |_| ComparisonOp::Re),
                map(ws(tag("=^")), |_| ComparisonOp::StartsWith),
                map(ws(tag("=$")), |_| ComparisonOp::EndsWith),
            )),
            ws(parse_string),
        )),
        |e| Filter::ComparisonExpr(e.0, e.1, e.2),
    )(i)
}

pub(crate) fn parse_existence_expr(i: &str) -> IResult<&str, Filter> {
    terminated(map(parse_path, Filter::ExistenceExpr), char('?'))(i)
}

pub(crate) fn parse_not_expr(i: &str) -> IResult<&str, Filter> {
    map(
        preceded(
            ws(char('!')),
            alt((parse_existence_expr, parse_grouped_expr, parse_not_expr)),
        ),
        |e| Filter::NotExpr(Box::new(e)),
    )(i)
}

pub(crate) fn parse_grouped_expr(i: &str) -> IResult<&str, Filter> {
    map(
        delimited(
            char('('),
            alt((parse_boolean_expr, parse_comparison_expr)),
            char(')'),
        ),
        |e| Filter::GroupedExpr(Box::new(e)),
    )(i)
}

fn parse_term_expr(i: &str) -> IResult<&str, Filter> {
    alt((
        parse_comparison_expr,
        parse_existence_expr,
        parse_grouped_expr,
        parse_not_expr,
    ))(i)
}

pub(crate) fn parse_boolean_expr(i: &str) -> IResult<&str, Filter> {
    let (i, (first, remainder)) = tuple((
        parse_term_expr,
        many0(tuple((
            alt((
                map(ws(tag("&&")), |_| BooleanOp::And),
                map(ws(tag("||")), |_| BooleanOp::Or),
            )),
            parse_term_expr,
        ))),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, (op, next)| {
            Filter::BooleanExpr(Box::new(prev), op, Box::new(next))
        }),
    ))
}

pub fn parse_filter(i: &str) -> IResult<&str, Filter> {
    all_consuming(alt((parse_boolean_expr, parse_term_expr)))(i)
}

pub fn parse_path(i: &str) -> IResult<&str, Path> {
    map(
        tuple((
            preceded(multispace0, parse_field_tag),
            opt(parse_field_occurrence),
            preceded(char('.'), parse_subfield_code),
            opt(delimited(
                char('['),
                map(digit1, |v: &str| v.parse::<usize>().unwrap()),
                char(']'),
            )),
            multispace0,
        )),
        |(tag, occurrence, code, index, _)| {
            Path::new(tag, occurrence, code, index)
        },
    )(i)
}

pub fn parse_path_list(i: &str) -> IResult<&str, Vec<Path>> {
    all_consuming(separated_list1(char(','), ws(parse_path)))(i)
}
