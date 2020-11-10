use crate::error::ParsePicaError;
use crate::parser::parse_field;
use crate::Subfield;
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Field {
    pub(crate) tag: String,
    pub(crate) occurrence: Option<String>,
    pub(crate) subfields: Vec<Subfield>,
}

impl Field {
    /// Create a new field.
    ///
    /// # Example
    /// ```
    /// use pica::Field;
    ///
    /// let field = Field::new("003@", None, vec![]);
    /// assert_eq!(field.tag(), "003@");
    /// assert_eq!(field.occurrence(), None);
    /// assert_eq!(field.subfields(), vec![]);
    /// ```
    pub fn new<S>(
        tag: S,
        occurrence: Option<S>,
        subfields: Vec<Subfield>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            tag: tag.into(),
            occurrence: occurrence.map(|o| o.into()),
            subfields,
        }
    }

    /// Returns the tag of the field.
    ///
    /// # Example
    /// ```
    /// use pica::Field;
    ///
    /// let field = Field::new("003@", None, vec![]);
    /// assert_eq!(field.tag(), "003@");
    /// ```
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the occurrence of the field.
    ///
    /// # Example
    /// ```
    /// use pica::Field;
    ///
    /// let field = Field::new("012A", Some("00"), vec![]);
    /// assert_eq!(field.occurrence(), Some("00"));
    /// ```
    pub fn occurrence(&self) -> Option<&str> {
        self.occurrence.as_deref()
    }

    /// Returns the subfields of the field.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Subfield};
    ///
    /// let field =
    ///     Field::new("012A", None, vec![Subfield::new('a', "123").unwrap()]);
    /// assert_eq!(field.subfields(), vec![Subfield::new('a', "123").unwrap()]);
    /// ```
    pub fn subfields(&self) -> &[Subfield] {
        &self.subfields
    }

    /// Returns the field as an pretty formatted string.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Subfield};
    ///
    /// let field = Field::new(
    ///     "012A",
    ///     None,
    ///     vec![
    ///         Subfield::new('a', "123").unwrap(),
    ///         Subfield::new('b', "456").unwrap(),
    ///     ],
    /// );
    /// assert_eq!(field.pretty(), "012A $a 123 $b 456");
    /// ```
    pub fn pretty(&self) -> String {
        let mut pretty_str = String::from(&self.tag);

        if let Some(occurrence) = self.occurrence() {
            pretty_str.push('/');
            pretty_str.push_str(&occurrence);
        }

        if !self.subfields.is_empty() {
            pretty_str.push(' ');
            pretty_str.push_str(
                &self
                    .subfields
                    .iter()
                    .map(|s| s.pretty())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }

        pretty_str
    }
}

impl FromStr for Field {
    type Err = ParsePicaError;

    /// Parse a pica+ encoded field.
    ///
    /// A Pica+ field constist of a tag, an optional occurrence and a list of
    /// [`Subfields`].
    ///
    /// # Grammar
    ///
    /// A field which conform to the following [EBNF] grammar will result in an
    /// [`Ok`] being returned.
    ///
    /// ```text
    /// Field      ::= Tag Occurrence? Subfield* '#x1e'
    /// Tag        ::= [012] [0-9]{2} ([A-Z] | '@')
    /// Occurrence ::= '/' [0-9]{2,3}
    /// Subfield   ::= Code Value
    /// Code       ::= [a-zA-Z0-9]
    /// Value      ::= [^#x1e#x1f]
    /// ```
    ///
    /// [EBNF]: https://www.w3.org/TR/REC-xml/#sec-notation
    ///
    /// # Example
    /// ```
    /// use pica::Field;
    ///
    /// assert!("003@ \u{1f}0123456789\u{1e}".parse::<Field>().is_ok());
    /// assert!("\u{1f}!123456789".parse::<Field>().is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_field(s) {
            Ok((_, field)) => Ok(field),
            _ => Err(ParsePicaError::InvalidField),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_new() {
        let field = Field::new("003@", None, vec![]);
        assert_eq!(field.tag(), "003@");
        assert_eq!(field.occurrence(), None);
        assert_eq!(field.subfields(), vec![]);
    }

    #[test]
    fn test_field_from_str() {
        let field = "012A/00 \u{1f}a123\u{1e}".parse::<Field>().unwrap();
        assert_eq!(field.tag(), "012A");
        assert_eq!(field.occurrence(), Some("00"));
        assert_eq!(field.subfields(), vec![Subfield::new('a', "123").unwrap()]);
    }
}
