use std::fmt;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::value;

use pica_core::ParseResult;

/// Boolean Operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BooleanOp {
    And, // and, "&&"
    Or,  // or, "||"
}

impl fmt::Display for BooleanOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
        }
    }
}

/// Comparison Operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonOp {
    Eq,         // equal, "=="
    Ne,         // not equal, "!="
    Gt,         // greater than, ">"
    Ge,         // greater than or equal, ">="
    Lt,         // less than, "<"
    Le,         // less than or equal, "<="
    StartsWith, // starts with, "=^"
    EndsWith,   // ends with, "=$"
    Similar,    // similar, "=*"
}

impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::StartsWith => write!(f, "=^"),
            Self::EndsWith => write!(f, "=$"),
            Self::Similar => write!(f, "=*"),
        }
    }
}

/// Parses comparison operator for byte strings.
pub(crate) fn parse_comparison_op_bstring(
    i: &[u8],
) -> ParseResult<ComparisonOp> {
    alt((
        value(ComparisonOp::Eq, tag("==")),
        value(ComparisonOp::Ne, tag("!=")),
        value(ComparisonOp::StartsWith, tag("=^")),
        value(ComparisonOp::EndsWith, tag("=$")),
        value(ComparisonOp::Similar, tag("=*")),
    ))(i)
}

/// Parses comparison operator for usize.
pub(crate) fn parse_comparison_op_usize(i: &[u8]) -> ParseResult<ComparisonOp> {
    alt((
        value(ComparisonOp::Eq, tag("==")),
        value(ComparisonOp::Ne, tag("!=")),
        value(ComparisonOp::Ge, tag(">=")),
        value(ComparisonOp::Gt, tag(">")),
        value(ComparisonOp::Le, tag("<=")),
        value(ComparisonOp::Lt, tag("<")),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestResult;

    #[test]
    fn test_parse_comparison_op() -> TestResult {
        // bstring
        assert_eq!(parse_comparison_op_bstring(b"==")?.1, ComparisonOp::Eq);
        assert_eq!(parse_comparison_op_bstring(b"!=")?.1, ComparisonOp::Ne);
        assert_eq!(
            parse_comparison_op_bstring(b"=^")?.1,
            ComparisonOp::StartsWith
        );
        assert_eq!(
            parse_comparison_op_bstring(b"=$")?.1,
            ComparisonOp::EndsWith
        );

        assert!(parse_comparison_op_bstring(b">=").is_err());
        assert!(parse_comparison_op_bstring(b">").is_err());
        assert!(parse_comparison_op_bstring(b"<=").is_err());
        assert!(parse_comparison_op_bstring(b"<").is_err());

        // usize
        assert_eq!(parse_comparison_op_usize(b"==")?.1, ComparisonOp::Eq);
        assert_eq!(parse_comparison_op_usize(b"!=")?.1, ComparisonOp::Ne);
        assert_eq!(parse_comparison_op_usize(b">=")?.1, ComparisonOp::Ge);
        assert_eq!(parse_comparison_op_usize(b">")?.1, ComparisonOp::Gt);
        assert_eq!(parse_comparison_op_usize(b"<=")?.1, ComparisonOp::Le);
        assert_eq!(parse_comparison_op_usize(b"<")?.1, ComparisonOp::Lt);

        assert!(parse_comparison_op_usize(b"=^").is_err());
        assert!(parse_comparison_op_usize(b"=$").is_err());
        assert!(parse_comparison_op_usize(b"=~").is_err());
        assert!(parse_comparison_op_usize(b"=*").is_err());

        Ok(())
    }

    #[test]
    fn test_boolean_op_to_string() -> TestResult {
        assert_eq!(BooleanOp::And.to_string(), "&&");
        assert_eq!(BooleanOp::Or.to_string(), "||");
        Ok(())
    }

    #[test]
    fn test_comparison_op_to_string() -> TestResult {
        assert_eq!(ComparisonOp::Eq.to_string(), "==");
        assert_eq!(ComparisonOp::Ne.to_string(), "!=");
        assert_eq!(ComparisonOp::Gt.to_string(), ">");
        assert_eq!(ComparisonOp::Ge.to_string(), ">=");
        assert_eq!(ComparisonOp::Lt.to_string(), "<");
        assert_eq!(ComparisonOp::Le.to_string(), "<=");
        assert_eq!(ComparisonOp::StartsWith.to_string(), "=^");
        assert_eq!(ComparisonOp::EndsWith.to_string(), "=$");
        assert_eq!(ComparisonOp::Similar.to_string(), "=*");
        Ok(())
    }
}
