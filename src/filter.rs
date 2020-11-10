use crate::error::ParsePicaError;
use crate::parser::parse_filter;
use crate::Path;
use std::str::FromStr;

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
pub enum Filter {
    ComparisonExpr(Path, ComparisonOp, String),
    ExistenceExpr(Path),
    BooleanExpr(Box<Filter>, BooleanOp, Box<Filter>),
    GroupedExpr(Box<Filter>),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let filter = "003@.0[0]?".parse::<Filter>().unwrap();
        assert_eq!(
            filter,
            Filter::ExistenceExpr(Path::new("003@", None, '0', Some(0)))
        );

        let result = "003@.0!".parse::<Filter>();
        assert_eq!(result.err(), Some(ParsePicaError::InvalidFilter));
    }
}
