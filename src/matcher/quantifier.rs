use std::fmt::{self, Display};

use winnow::combinator::alt;
use winnow::{ModalResult, Parser};

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

#[inline]
pub(crate) fn parse_quantifier(
    i: &mut &[u8],
) -> ModalResult<Quantifier> {
    alt(("ALL".value(Quantifier::All), "ANY".value(Quantifier::Any)))
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quantifier() {
        use Quantifier::*;

        assert_eq!(parse_quantifier.parse(b"ALL").unwrap(), All);
        assert_eq!(parse_quantifier.parse(b"ANY").unwrap(), Any);
    }
}
