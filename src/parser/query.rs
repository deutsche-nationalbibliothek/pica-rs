use crate::parser::parse_path;
use crate::{ComparisonOp, LogicalOp, Query};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, none_of, space0};
use nom::combinator::{map, recognize, verify};
use nom::multi::many1;
use nom::sequence::{delimited, pair, preceded};
use nom::IResult;

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
    use crate::Path;

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
}
