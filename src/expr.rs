use crate::error::ParsePicaError;
use crate::parser::parse_expr;
use crate::Path;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Op {
    Eq,
    Ne,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Predicate(Path, Op, String),
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
