use std::fmt::{self, Display};

/// Relational Operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalOp {
    Equal,              // equal, "=="
    NotEqual,           // not equal, "!="
    GreaterThan,        // greater than, ">"
    GreaterThanOrEqual, // greater than or equal, ">="
    LessThan,           // less than, "<"
    LessThanOrEqual,    // less than or equal, "<="
    StartsWith,         // starts with, "=^"
    StartsNotWith,      // starts not with, "!^"
    EndsWith,           // ends with, "=$"
    EndsNotWith,        // ends not with, "!$"
    Similar,            // similar, "=*"
    Contains,           // contains, "=?"
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
    /// assert!(RelationalOp::Equal.is_usize_applicable());
    /// assert!(RelationalOp::NotEqual.is_usize_applicable());
    /// assert!(RelationalOp::GreaterThanOrEqual.is_usize_applicable());
    /// assert!(RelationalOp::GreaterThan.is_usize_applicable());
    /// assert!(RelationalOp::LessThanOrEqual.is_usize_applicable());
    /// assert!(RelationalOp::LessThan.is_usize_applicable());
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
            RelationalOp::Equal
                | RelationalOp::NotEqual
                | RelationalOp::GreaterThanOrEqual
                | RelationalOp::GreaterThan
                | RelationalOp::LessThan
                | RelationalOp::LessThanOrEqual
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
    /// assert!(RelationalOp::Contains.is_str_applicable());
    /// assert!(RelationalOp::EndsNotWith.is_str_applicable());
    /// assert!(RelationalOp::EndsWith.is_str_applicable());
    /// assert!(RelationalOp::Equal.is_str_applicable());
    /// assert!(RelationalOp::NotEqual.is_str_applicable());
    /// assert!(RelationalOp::Similar.is_str_applicable());
    /// assert!(RelationalOp::StartsNotWith.is_str_applicable());
    /// assert!(RelationalOp::StartsWith.is_str_applicable());
    ///
    /// assert!(!RelationalOp::GreaterThanOrEqual.is_str_applicable());
    /// assert!(!RelationalOp::GreaterThan.is_str_applicable());
    /// assert!(!RelationalOp::LessThanOrEqual.is_str_applicable());
    /// assert!(!RelationalOp::LessThan.is_str_applicable());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_str_applicable(&self) -> bool {
        !matches!(
            self,
            RelationalOp::GreaterThanOrEqual
                | RelationalOp::GreaterThan
                | RelationalOp::LessThan
                | RelationalOp::LessThanOrEqual
        )
    }
}

impl Display for RelationalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RelationalOp::Equal => write!(f, "=="),
            RelationalOp::NotEqual => write!(f, "!="),
            RelationalOp::GreaterThan => write!(f, ">"),
            RelationalOp::GreaterThanOrEqual => write!(f, ">="),
            RelationalOp::LessThan => write!(f, "<"),
            RelationalOp::LessThanOrEqual => write!(f, "<="),
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
            Self::Equal,
            Self::NotEqual,
            Self::GreaterThan,
            Self::GreaterThanOrEqual,
            Self::LessThan,
            Self::LessThanOrEqual,
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
