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
