//! Pica+ Subfield

use crate::error::ParsePicaError;
use crate::parser::parse_subfield;
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Subfield {
    pub(crate) code: char,
    pub(crate) value: String,
}

impl Subfield {
    pub fn new<S>(code: char, value: S) -> Result<Self, ParsePicaError>
    where
        S: Into<String>,
    {
        if code.is_ascii_alphanumeric() {
            Ok(Subfield {
                code,
                value: value.into(),
            })
        } else {
            Err(ParsePicaError::InvalidSubfield)
        }
    }

    pub(crate) fn from_unchecked<S>(code: char, value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            code,
            value: value.into(),
        }
    }

    pub fn code(&self) -> char {
        self.code
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn pretty(&self) -> String {
        format!("${} {}", self.code, self.value)
    }
}

impl FromStr for Subfield {
    type Err = ParsePicaError;

    /// Parse a pica+ encoded subfield.
    ///
    /// A Pica+ subfield constist of a alpha-numerical subfield code and a
    /// value (string literal). The subfield is preceded by a unit separator
    /// (`\x1f`).
    ///
    /// # Grammar
    ///
    /// All subfields which conform to the following [EBNF] grammar will result
    /// in an [`Ok`] being returned.
    ///
    /// ```text
    /// Subfield ::= Code Value
    /// Code     ::= [a-zA-Z0-9]
    /// Value    ::= [^#x1e#x1f]
    /// ```
    ///
    /// [EBNF]: https://www.w3.org/TR/REC-xml/#sec-notation
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_subfield(s) {
            Ok((_, subfield)) => Ok(subfield),
            _ => Err(ParsePicaError::InvalidSubfield),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subfield_unchecked() {
        let subfield = Subfield::from_unchecked('!', String::new());
        assert_eq!(subfield.code(), '!');
    }
}
