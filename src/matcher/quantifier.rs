use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Quantifier {
    All,
    #[default]
    Any,
}

impl Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "ALL"),
            Self::Any => write!(f, "ANY"),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Quantifier {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match g.choose(&[true, false]).unwrap() {
            true => Quantifier::All,
            false => Quantifier::Any,
        }
    }
}
