use crate::error::ParsePicaError;
use crate::parser::parse_query;
use crate::Path;
use std::boxed::Box;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum ComparisonOp {
    Eq,
    Ne,
}

#[derive(Debug, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum Query {
    Predicate(Path, ComparisonOp, String),
    Connective(Box<Query>, LogicalOp, Box<Query>),
    Parens(Box<Query>),
}

impl FromStr for Query {
    type Err = ParsePicaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_query(s) {
            Ok((_, path)) => Ok(path),
            _ => Err(ParsePicaError::InvalidQuery),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_from_str() {
        let query: Query = "003@.0 == 123456789X".parse().unwrap();
        assert_eq!(
            query,
            Query::Predicate(
                Path::new("003@", "", '0'),
                ComparisonOp::Eq,
                "123456789X".to_string()
            )
        );

        let query: Query = "003@.0 != 123456789X".parse().unwrap();
        assert_eq!(
            query,
            Query::Predicate(
                Path::new("003@", "", '0'),
                ComparisonOp::Ne,
                "123456789X".to_string()
            )
        );

        let query: Query =
            "003@.0 == 123456789X && 012A.a == a".parse().unwrap();
        let lhs = Query::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123456789X".to_string(),
        );
        let rhs = Query::Predicate(
            Path::new("012A", "", 'a'),
            ComparisonOp::Eq,
            "a".to_string(),
        );

        assert_eq!(
            query,
            Query::Connective(Box::new(lhs), LogicalOp::And, Box::new(rhs))
        );

        let query: Query =
            "003@.0 == 123456789X || 012A.a == a".parse().unwrap();
        let lhs = Query::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123456789X".to_string(),
        );
        let rhs = Query::Predicate(
            Path::new("012A", "", 'a'),
            ComparisonOp::Eq,
            "a".to_string(),
        );

        assert_eq!(
            query,
            Query::Connective(Box::new(lhs), LogicalOp::Or, Box::new(rhs))
        );

        let query: Query =
            "(003@.0 == 123456789X && 012A.a == a) || 012A.a == b"
                .parse()
                .unwrap();
        let p1 = Query::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123456789X".to_string(),
        );
        let p2 = Query::Predicate(
            Path::new("012A", "", 'a'),
            ComparisonOp::Eq,
            "a".to_string(),
        );
        let p3 = Query::Predicate(
            Path::new("012A", "", 'a'),
            ComparisonOp::Eq,
            "b".to_string(),
        );

        assert_eq!(
            query,
            Query::Connective(
                Box::new(Query::Parens(Box::new(Query::Connective(
                    Box::new(p1),
                    LogicalOp::And,
                    Box::new(p2)
                )))),
                LogicalOp::Or,
                Box::new(p3)
            )
        );

        assert_eq!(
            "(003@.0 == 123456789X(".parse::<Query>().err(),
            Some(ParsePicaError::InvalidQuery)
        );
    }
}
