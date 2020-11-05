use crate::error::ParsePicaError;
use crate::parser::parse_expr;
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
pub enum Expr {
    Predicate(Path, ComparisonOp, String),
    Connective(Box<Expr>, LogicalOp, Box<Expr>),
    Parens(Box<Expr>),
}

impl FromStr for Expr {
    type Err = ParsePicaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_expr(s) {
            Ok((_, path)) => Ok(path),
            _ => Err(ParsePicaError::InvalidExpr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_from_str() {
        let expr: Expr = "003@.0 == 123456789X".parse().unwrap();
        assert_eq!(
            expr,
            Expr::Predicate(
                Path::new("003@", "", '0'),
                ComparisonOp::Eq,
                "123456789X".to_string()
            )
        );

        let expr: Expr = "003@.0 != 123456789X".parse().unwrap();
        assert_eq!(
            expr,
            Expr::Predicate(
                Path::new("003@", "", '0'),
                ComparisonOp::Ne,
                "123456789X".to_string()
            )
        );

        let expr: Expr = "003@.0 == 123456789X && 012A.a == a".parse().unwrap();
        let lhs = Expr::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123456789X".to_string(),
        );
        let rhs = Expr::Predicate(
            Path::new("012A", "", 'a'),
            ComparisonOp::Eq,
            "a".to_string(),
        );

        assert_eq!(
            expr,
            Expr::Connective(Box::new(lhs), LogicalOp::And, Box::new(rhs))
        );

        let expr: Expr = "003@.0 == 123456789X || 012A.a == a".parse().unwrap();
        let lhs = Expr::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123456789X".to_string(),
        );
        let rhs = Expr::Predicate(
            Path::new("012A", "", 'a'),
            ComparisonOp::Eq,
            "a".to_string(),
        );

        assert_eq!(
            expr,
            Expr::Connective(Box::new(lhs), LogicalOp::Or, Box::new(rhs))
        );

        let expr: Expr = "(003@.0 == 123456789X && 012A.a == a) || 012A.a == b"
            .parse()
            .unwrap();
        let p1 = Expr::Predicate(
            Path::new("003@", "", '0'),
            ComparisonOp::Eq,
            "123456789X".to_string(),
        );
        let p2 = Expr::Predicate(
            Path::new("012A", "", 'a'),
            ComparisonOp::Eq,
            "a".to_string(),
        );
        let p3 = Expr::Predicate(
            Path::new("012A", "", 'a'),
            ComparisonOp::Eq,
            "b".to_string(),
        );

        assert_eq!(
            expr,
            Expr::Connective(
                Box::new(Expr::Parens(Box::new(Expr::Connective(
                    Box::new(p1),
                    LogicalOp::And,
                    Box::new(p2)
                )))),
                LogicalOp::Or,
                Box::new(p3)
            )
        );
    }
}
