//! Pica+ Subfield

use crate::error::ParsePicaError;
use crate::parser::parse_subfield;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct Subfield {
    pub(crate) code: char,
    pub(crate) value: String,
}

impl Subfield {
    /// Creates a new subfield without checking that the code is valid.
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::from_unchecked('a', "123456789");
    /// assert_eq!(subfield.code(), 'a');
    /// assert_eq!(subfield.value(), "123456789");
    /// ```
    pub fn from_unchecked<S: Into<String>>(code: char, value: S) -> Self {
        Self {
            code,
            value: value.into(),
        }
    }

    /// Returns the code of the subfield.
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = "\u{1f}a123".parse::<Subfield>().unwrap();
    /// assert_eq!(subfield.code(), 'a');
    /// ```
    pub fn code(&self) -> char {
        self.code
    }

    /// Returns the value of the subfield.
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = "\u{1f}a123".parse::<Subfield>().unwrap();
    /// assert_eq!(subfield.value(), "123");
    /// ```
    pub fn value(&self) -> &String {
        &self.value
    }
}

impl FromStr for Subfield {
    type Err = ParsePicaError;

    /// Parse a pica+ encoded string.
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
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// assert!("\u{1f}0123456789".parse::<Subfield>().is_ok());
    /// assert!("\u{1f}!123456789".parse::<Subfield>().is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_subfield(s) {
            Ok((_, subfield)) => Ok(subfield),
            _ => Err(ParsePicaError::InvalidSubfield),
        }
    }
}
