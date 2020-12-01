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
use crate::string::parse_string;
use crate::{Field, Filter, Path, Record, Subfield};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{
    char, digit1, multispace0, none_of, one_of, satisfy,
};
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::error::ParseError;
use nom::multi::{count, many0, many1, many_m_n, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

/// Strip whitespaces from the beginning and end.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn parse_subfield_code(i: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

pub fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_code, parse_subfield_value),
            |(code, value)| Subfield::from_unchecked(code, value),
        ),
    )(i)
}

fn parse_field_tag(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        one_of("012"),
        count(one_of("0123456789"), 2),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
    )))(i)
}

fn parse_field_occurrence(i: &str) -> IResult<&str, &str> {
    preceded(char('/'), recognize(many_m_n(2, 3, one_of("0123456789"))))(i)
}

pub fn parse_field(i: &str) -> IResult<&str, Field> {
    terminated(
        map(
            pair(
                pair(parse_field_tag, opt(parse_field_occurrence)),
                preceded(char(' '), many0(parse_subfield)),
            ),
            |((tag, occurrence), subfields)| {
                Field::new(tag, occurrence, subfields)
            },
        ),
        char('\u{1e}'),
    )(i)
}

pub fn parse_record(i: &str) -> IResult<&str, Record> {
    all_consuming(map(many1(parse_field), Record::new))(i)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Path;

    #[test]
    fn test_parse_subfield_code() {
        for range in vec!['a'..='z', 'A'..='Z', '0'..='9'] {
            for c in range {
                assert_eq!(parse_subfield_code(&String::from(c)), Ok(("", c)));
            }
        }
    }

    #[test]
    fn test_parse_subfield_value() {
        assert_eq!(parse_subfield_value(""), Ok(("", "")));
        assert_eq!(parse_subfield_value("abc"), Ok(("", "abc")));
        assert_eq!(parse_subfield_value("ab\u{1f}c"), Ok(("\u{1f}c", "ab")));
        assert_eq!(parse_subfield_value("ab\u{1e}c"), Ok(("\u{1e}c", "ab")));
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield("\u{1f}a123"),
            Ok(("", Subfield::from_unchecked('a', "123")))
        );
        assert!(parse_subfield("!a123").is_err());
    }

    #[test]
    fn test_field_tag() {
        for tag in vec!["000A", "100A", "200A", "000A", "000@"] {
            assert_eq!(parse_field_tag(tag), Ok(("", tag)));
        }

        for tag in vec!["300A", "0A0A", "00AA", "0001"] {
            assert!(parse_field_tag(tag).is_err())
        }
    }

    #[test]
    fn test_parse_field_occurrence() {
        assert_eq!(parse_field_occurrence("/00"), Ok(("", "00")));
        assert_eq!(parse_field_occurrence("/001"), Ok(("", "001")));
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field("012A/00 \u{1e}"),
            Ok(("", Field::new("012A", Some("00"), vec![])))
        );
        assert_eq!(
            parse_field("012A \u{1e}"),
            Ok(("", Field::new("012A", None, vec![])))
        );
        assert_eq!(
            parse_field("003@ \u{1f}0123456789\u{1e}"),
            Ok((
                "",
                Field::new(
                    "003@",
                    None,
                    vec![Subfield::from_unchecked('0', "123456789")]
                )
            ))
        );
    }

    #[test]
    fn test_parse_record() {
        assert_eq!(
            parse_record("003@ \u{1f}0123456789\u{1e}"),
            Ok((
                "",
                Record::new(vec![Field::new(
                    "003@",
                    None,
                    vec![Subfield::new('0', "123456789").unwrap()]
                )])
            ))
        );
    }

    #[test]
    fn test_parse_comparison_expr() {
        let path = Path::new("003@", None, '0', None);
        let value = "123456789X".to_string();
        let expected = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);
        assert_eq!(
            parse_comparison_expr("003@.0 == '123456789X'"),
            Ok(("", expected))
        );

        let path = Path::new("003@", None, '0', None);
        let value = "123456789X".to_string();
        let expected = Filter::ComparisonExpr(path, ComparisonOp::Ne, value);
        assert_eq!(
            parse_comparison_expr("003@.0 != '123456789X'"),
            Ok(("", expected))
        );

        let path = Path::new("028@", None, 'd', Some(0));
        let value = "abc".to_string();
        let expected = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);
        assert_eq!(
            parse_comparison_expr("028@.d[0] == 'abc'"),
            Ok(("", expected))
        );

        let path = Path::new("002@", None, '0', None);
        let value = "Tp[12]".to_string();
        let expected = Filter::ComparisonExpr(path, ComparisonOp::Re, value);
        assert_eq!(
            parse_comparison_expr("002@.0 =~ 'Tp[12]'"),
            Ok(("", expected))
        );

        let path = Path::new("002@", None, '0', None);
        let value = "Tp".to_string();
        let expected =
            Filter::ComparisonExpr(path, ComparisonOp::StartsWith, value);
        assert_eq!(parse_comparison_expr("002@.0 =^ 'Tp'"), Ok(("", expected)));

        let path = Path::new("002@", None, '0', None);
        let value = "Tp".to_string();
        let expected =
            Filter::ComparisonExpr(path, ComparisonOp::EndsWith, value);
        assert_eq!(parse_comparison_expr("002@.0 =$ 'Tp'"), Ok(("", expected)));
    }

    #[test]
    fn test_parse_boolean_expr() {
        let term1 = Filter::ComparisonExpr(
            Path::new("003@", None, '0', None),
            ComparisonOp::Eq,
            "123456789X".to_string(),
        );

        let path = Path::new("002@", None, '0', None);
        let value = "Tp1".to_string();
        let term2 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("012A", None, '0', None);
        let value = "foo".to_string();
        let term3 = Filter::ComparisonExpr(path, ComparisonOp::Ne, value);

        let expected = Filter::BooleanExpr(
            Box::new(Filter::BooleanExpr(
                Box::new(term1),
                BooleanOp::Or,
                Box::new(term2),
            )),
            BooleanOp::And,
            Box::new(term3),
        );
        assert_eq!(
            parse_boolean_expr(
                "003@.0 == '123456789X' || 002@.0 == 'Tp1' && 012A.0 != 'foo'"
            ),
            Ok(("", expected))
        );

        let path = Path::new("003@", None, '0', None);
        let value = "123456789X".to_string();
        let term1 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("002@", None, '0', None);
        let value = "Tp1".to_string();
        let term2 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("012A", None, '0', None);
        let value = "foo".to_string();
        let term3 = Filter::ComparisonExpr(path, ComparisonOp::Ne, value);

        let expected = Filter::BooleanExpr(
            Box::new(Filter::GroupedExpr(Box::new(Filter::BooleanExpr(
                Box::new(term1),
                BooleanOp::Or,
                Box::new(term2),
            )))),
            BooleanOp::And,
            Box::new(term3),
        );
        assert_eq!(
            parse_boolean_expr(
                "(003@.0 == '123456789X' || 002@.0 == 'Tp1') && 012A.0 != 'foo'"
            ),
            Ok(("", expected))
        );

        let path = Path::new("003@", None, '0', None);
        let value = "123456789X".to_string();
        let term1 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("002@", None, '0', None);
        let value = "Tp1".to_string();
        let term2 = Filter::ComparisonExpr(path, ComparisonOp::Eq, value);

        let path = Path::new("012A", None, '0', None);
        let value = "foo".to_string();
        let term3 = Filter::ComparisonExpr(path, ComparisonOp::Ne, value);

        let expected = Filter::BooleanExpr(
            Box::new(term1),
            BooleanOp::Or,
            Box::new(Filter::GroupedExpr(Box::new(Filter::BooleanExpr(
                Box::new(term2),
                BooleanOp::And,
                Box::new(term3),
            )))),
        );
        assert_eq!(
            parse_boolean_expr(
                "003@.0 == '123456789X' || (002@.0 == 'Tp1' && 012A.0 != 'foo')"
            ),
            Ok(("", expected))
        );
    }

    fn parse_helper(i: &str) -> IResult<&str, String> {
        parse_string(i)
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_helper("'abc'"), Ok(("", String::from("abc"))));
        assert_eq!(parse_helper("'a\tb'"), Ok(("", String::from("a\tb"))));
        assert_eq!(parse_helper("'\u{1f}'"), Ok(("", String::from("\u{1f}"))));
    }
}
