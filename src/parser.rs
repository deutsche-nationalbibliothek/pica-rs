use crate::{ComparisonOp, Field, LogicalOp, Path, Query, Record, Subfield};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, none_of, one_of, space0};
use nom::combinator::{all_consuming, map, opt, recognize, verify};
use nom::multi::{count, many0, many1, many_m_n};
use nom::sequence::{
    delimited, pair, preceded, separated_pair, terminated, tuple,
};
use nom::IResult;
use std::boxed::Box;

pub(crate) fn parse_subfield_code(i: &str) -> IResult<&str, char> {
    alt((
        one_of("abcdefghijklmnopqrstuvwxyz"),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        one_of("0123456789"),
    ))(i)
}

pub(crate) fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

pub(crate) fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_code, parse_subfield_value),
            |(code, value)| Subfield::from_unchecked(code, value),
        ),
    )(i)
}

pub(crate) fn parse_field_tag(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        one_of("012"),
        count(one_of("0123456789"), 2),
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ@"),
    )))(i)
}

pub(crate) fn parse_field_occurrence(i: &str) -> IResult<&str, &str> {
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
                Field::new(tag, occurrence.unwrap_or_default(), subfields)
            },
        ),
        char('\u{1e}'),
    )(i)
}

pub fn parse_record(i: &str) -> IResult<&str, Record> {
    all_consuming(map(many1(parse_field), Record::new))(i)
}

pub fn parse_path(i: &str) -> IResult<&str, Path> {
    map(
        delimited(
            space0,
            separated_pair(
                pair(parse_field_tag, opt(parse_field_occurrence)),
                nom::character::complete::char('.'),
                parse_subfield_code,
            ),
            space0,
        ),
        |((tag, occurrence), code)| {
            Path::new(tag, occurrence.unwrap_or_default(), code)
        },
    )(i)
}

pub(crate) fn parse_comparison_op(i: &str) -> IResult<&str, ComparisonOp> {
    delimited(
        space0,
        alt((
            map(tag("=="), |_| ComparisonOp::Eq),
            map(tag("!="), |_| ComparisonOp::Ne),
        )),
        space0,
    )(i)
}

pub(crate) fn parse_logical_op(i: &str) -> IResult<&str, LogicalOp> {
    delimited(
        space0,
        alt((
            map(tag("&&"), |_| LogicalOp::And),
            map(tag("||"), |_| LogicalOp::Or),
        )),
        space0,
    )(i)
}

pub(crate) fn parse_literal(i: &str) -> IResult<&str, &str> {
    delimited(
        space0,
        verify(recognize(many1(none_of("\"\\ ()"))), |s: &str| {
            !s.is_empty()
        }),
        space0,
    )(i)
}

pub(crate) fn parse_predicate(i: &str) -> IResult<&str, Query> {
    map(
        pair(pair(parse_path, parse_comparison_op), parse_literal),
        |((path, op), literal)| {
            Query::Predicate(path, op, String::from(literal))
        },
    )(i)
}

pub(crate) fn parse_connective(i: &str) -> IResult<&str, Query> {
    delimited(
        space0,
        map(
            pair(
                pair(alt((parse_predicate, parse_parens)), parse_logical_op),
                alt((parse_predicate, parse_parens)),
            ),
            |((lhs, op), rhs)| {
                Query::Connective(Box::new(lhs), op, Box::new(rhs))
            },
        ),
        space0,
    )(i)
}

pub(crate) fn parse_parens(i: &str) -> IResult<&str, Query> {
    map(
        delimited(
            preceded(space0, char('(')),
            preceded(space0, parse_query),
            preceded(space0, char(')')),
        ),
        |query| Query::Parens(Box::new(query)),
    )(i)
}

pub fn parse_query(i: &str) -> IResult<&str, Query> {
    delimited(
        space0,
        alt((parse_connective, parse_parens, parse_predicate)),
        space0,
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comparison_op() {
        assert_eq!(parse_comparison_op("  == "), Ok(("", ComparisonOp::Eq)));
        assert_eq!(parse_comparison_op("  != "), Ok(("", ComparisonOp::Ne)));
    }
    #[test]
    fn test_parse_logical_op() {
        assert_eq!(parse_logical_op("  && "), Ok(("", LogicalOp::And)));
        assert_eq!(parse_logical_op("  || "), Ok(("", LogicalOp::Or)));
    }

    #[test]
    fn test_parse_literal() {
        assert_eq!(parse_literal(" 0123456789X "), Ok(("", "0123456789X")));
    }

    #[test]
    fn test_parse_predicate() {
        assert_eq!(
            parse_query("003@.0 == 123"),
            Ok((
                "",
                Query::Predicate(
                    Path::new("003@", "", '0'),
                    ComparisonOp::Eq,
                    "123".to_string()
                )
            ))
        );
        assert_eq!(
            parse_query("012A/00.0 != 123"),
            Ok((
                "",
                Query::Predicate(
                    Path::new("012A", "00", '0'),
                    ComparisonOp::Ne,
                    "123".to_string()
                )
            ))
        );
    }

    #[test]
    fn test_parse_connective() {
        let lhs = Box::new(Query::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123".to_string(),
        ));
        let rhs = Box::new(Query::Predicate(
            Path::new("002@", "", '0'),
            ComparisonOp::Ne,
            "Tp1".to_string(),
        ));

        assert_eq!(
            parse_query("003@.0 == 123 || 002@.0 != Tp1"),
            Ok(("", Query::Connective(lhs, LogicalOp::Or, rhs)))
        );
    }

    #[test]
    fn test_parse_parens() {
        let p1 = Box::new(Query::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123".to_string(),
        ));
        let p2 = Box::new(Query::Predicate(
            Path::new("002@", "", '0'),
            ComparisonOp::Eq,
            "Tp3".to_string(),
        ));

        assert_eq!(
            parse_parens("(003@.0 == 123 && 002@.0 == Tp3)"),
            Ok((
                "",
                Query::Parens(Box::new(Query::Connective(
                    p1,
                    LogicalOp::And,
                    p2
                )))
            ))
        );
    }

    #[test]
    fn test_parse_query() {
        let p1 = Box::new(Query::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123".to_string(),
        ));
        let p2 = Box::new(Query::Predicate(
            Path::new("002@", "", '0'),
            ComparisonOp::Eq,
            "Tp3".to_string(),
        ));
        let p3 = Box::new(Query::Predicate(
            Path::new("002@", "", '0'),
            ComparisonOp::Eq,
            "Tf1".to_string(),
        ));

        assert_eq!(
            parse_query("(003@.0 == 123 && 002@.0 == Tp3) || 002@.0 == Tf1"),
            Ok((
                "",
                Query::Connective(
                    Box::new(Query::Parens(Box::new(Query::Connective(
                        p1,
                        LogicalOp::And,
                        p2
                    )))),
                    LogicalOp::Or,
                    p3
                )
            ))
        );
    }

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path("   003@.0 "),
            Ok(("", Path::new("003@", "", '0')))
        );
        assert_eq!(
            parse_path("   003@/00.0 "),
            Ok(("", Path::new("003@", "00", '0')))
        );
    }

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
            Ok(("", Field::new("012A", "00", vec![])))
        );
        assert_eq!(
            parse_field("012A \u{1e}"),
            Ok(("", Field::new("012A", "", vec![])))
        );
        assert_eq!(
            parse_field("003@ \u{1f}0123456789\u{1e}"),
            Ok((
                "",
                Field::new(
                    "003@",
                    "",
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
                    "",
                    vec![Subfield::new('0', "123456789").unwrap()]
                )])
            ))
        );
    }
}
