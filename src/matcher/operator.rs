use std::fmt::{self, Display};

use winnow::combinator::alt;
use winnow::{ModalResult, Parser};

/// Relational Operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalOp {
    Eq,            // equal, "=="
    Ne,            // not equal, "!="
    Gt,            // greater than, ">"
    Ge,            // greater than or equal, ">="
    Lt,            // less than, "<"
    Le,            // less than or equal, "<="
    StartsWith,    // starts with, "=^"
    StartsNotWith, // starts not with, "!^"
    EndsWith,      // ends with, "=$"
    EndsNotWith,   // ends not with, "!$"
    Similar,       // similar, "=*"
    Contains,      // contains, "=?"
}

impl RelationalOp {
    /// Returns true of the operator can be used in combination with the
    /// `usize` type, otherwise false.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::RelationalOp;
    ///
    /// assert!(RelationalOp::Eq.is_usize_applicable());
    /// assert!(RelationalOp::Ne.is_usize_applicable());
    /// assert!(RelationalOp::Ge.is_usize_applicable());
    /// assert!(RelationalOp::Gt.is_usize_applicable());
    /// assert!(RelationalOp::Le.is_usize_applicable());
    /// assert!(RelationalOp::Lt.is_usize_applicable());
    ///
    /// assert!(!RelationalOp::Contains.is_usize_applicable());
    /// assert!(!RelationalOp::EndsNotWith.is_usize_applicable());
    /// assert!(!RelationalOp::EndsWith.is_usize_applicable());
    /// assert!(!RelationalOp::Similar.is_usize_applicable());
    /// assert!(!RelationalOp::StartsNotWith.is_usize_applicable());
    /// assert!(!RelationalOp::StartsWith.is_usize_applicable());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_usize_applicable(&self) -> bool {
        matches!(
            self,
            RelationalOp::Eq
                | RelationalOp::Ne
                | RelationalOp::Ge
                | RelationalOp::Gt
                | RelationalOp::Lt
                | RelationalOp::Le
        )
    }

    /// Returns true of the operator can be used in combination with
    /// `str` or byte slices, otherwise false.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::RelationalOp;
    ///
    /// assert!(RelationalOp::Eq.is_str_applicable());
    /// assert!(RelationalOp::Ne.is_str_applicable());
    /// assert!(RelationalOp::Contains.is_str_applicable());
    /// assert!(RelationalOp::EndsNotWith.is_str_applicable());
    /// assert!(RelationalOp::EndsWith.is_str_applicable());
    /// assert!(RelationalOp::Similar.is_str_applicable());
    /// assert!(RelationalOp::StartsNotWith.is_str_applicable());
    /// assert!(RelationalOp::StartsWith.is_str_applicable());
    ///
    /// assert!(!RelationalOp::Ge.is_str_applicable());
    /// assert!(!RelationalOp::Gt.is_str_applicable());
    /// assert!(!RelationalOp::Le.is_str_applicable());
    /// assert!(!RelationalOp::Lt.is_str_applicable());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_str_applicable(&self) -> bool {
        !matches!(
            self,
            RelationalOp::Ge
                | RelationalOp::Gt
                | RelationalOp::Lt
                | RelationalOp::Le
        )
    }
}

impl Display for RelationalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RelationalOp::Eq => write!(f, "=="),
            RelationalOp::Ne => write!(f, "!="),
            RelationalOp::Gt => write!(f, ">"),
            RelationalOp::Ge => write!(f, ">="),
            RelationalOp::Lt => write!(f, "<"),
            RelationalOp::Le => write!(f, "<="),
            RelationalOp::StartsWith => write!(f, "=^"),
            RelationalOp::StartsNotWith => write!(f, "!^"),
            RelationalOp::EndsWith => write!(f, "=$"),
            RelationalOp::EndsNotWith => write!(f, "!$"),
            RelationalOp::Similar => write!(f, "=*"),
            RelationalOp::Contains => write!(f, "=?"),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for RelationalOp {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        g.choose(&[
            Self::Eq,
            Self::Ne,
            Self::Gt,
            Self::Ge,
            Self::Lt,
            Self::Le,
            Self::StartsWith,
            Self::StartsNotWith,
            Self::EndsWith,
            Self::EndsNotWith,
            Self::Similar,
            Self::Contains,
        ])
        .unwrap()
        .clone()
    }
}

/// Parse RelationalOp which can be used for string comparisons.
#[inline]
pub(crate) fn parse_relational_operator(
    i: &mut &[u8],
) -> ModalResult<RelationalOp> {
    use RelationalOp::*;

    alt((
        "==".value(Eq),
        "!=".value(Ne),
        "=^".value(StartsWith),
        "!^".value(StartsNotWith),
        "=$".value(EndsWith),
        "!$".value(EndsNotWith),
        "=*".value(Similar),
        "=?".value(Contains),
        ">=".value(Ge),
        ">".value(Gt),
        "<=".value(Le),
        "<".value(Lt),
    ))
    .parse_next(i)
}

/// Boolean Operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BooleanOp {
    And, // and, "&&"
    Or,  // or, "||"
    Xor, // xor, "^"
}

impl Display for BooleanOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::Xor => write!(f, "^"),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for BooleanOp {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        g.choose(&[Self::And, Self::Or, Self::Xor]).unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relational_operator() {
        use RelationalOp::*;

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_relational_operator
                        .parse($input.as_bytes())
                        .unwrap(),
                    $expected
                );
            };
        }

        parse_success!("==", Eq);
        parse_success!("!=", Ne);
        parse_success!(">=", Ge);
        parse_success!(">", Gt);
        parse_success!("<=", Le);
        parse_success!("<", Lt);
        parse_success!("=^", StartsWith);
        parse_success!("!^", StartsNotWith);
        parse_success!("=$", EndsWith);
        parse_success!("!$", EndsNotWith);
        parse_success!("=*", Similar);
        parse_success!("=?", Contains);
    }
}
