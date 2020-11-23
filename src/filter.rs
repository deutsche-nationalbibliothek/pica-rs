use crate::error::ParsePicaError;
use crate::parser::parse_filter;
use crate::Path;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Re,
    StartsWith,
    EndsWith,
}

#[derive(Debug, PartialEq)]
pub enum BooleanOp {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum Filter {
    ComparisonExpr(Path, ComparisonOp, String),
    ExistenceExpr(Path),
    BooleanExpr(Box<Filter>, BooleanOp, Box<Filter>),
    GroupedExpr(Box<Filter>),
    NotExpr(Box<Filter>),
}

impl FromStr for Filter {
    type Err = ParsePicaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_filter(s) {
            Ok((_, path)) => Ok(path),
            _ => Err(ParsePicaError::InvalidFilter),
        }
    }
}
