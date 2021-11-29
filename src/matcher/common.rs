use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::value;

use crate::common::ParseResult;

/// Boolean Operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BooleanOp {
    And, // and, "&&"
    Or,  // or, "||"
}

/// Comparison Operators
#[derive(Debug, Clone, PartialEq)]
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
    use crate::test::TestResult;

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
}
