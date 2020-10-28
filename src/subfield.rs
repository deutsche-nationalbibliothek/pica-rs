use crate::{error::PicaParseError, parser::parse_subfield};
use std::{fmt, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub struct Subfield {
    code: char,
    value: String,
}

impl Subfield {
    /// Create a new subfield
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('a', "foo");
    /// assert_eq!(subfield.code(), 'a');
    /// ```
    pub fn new<S>(code: char, value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            code,
            value: value.into(),
        }
    }

    /// Returns the subfield code.
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('a', "foo");
    /// assert_eq!(subfield.code(), 'a');
    /// ```
    pub fn code(&self) -> char {
        self.code
    }

    /// Returns the subfield value.
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('a', "foo");
    /// assert_eq!(subfield.value(), "foo");
    /// ```
    pub fn value(&self) -> &String {
        &self.value
    }
}

impl fmt::Display for Subfield {
    /// Format a subfield.
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('b', "bar");
    /// assert_eq!(subfield.to_string(), "$b bar");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${} {}", self.code, self.value)
    }
}

impl FromStr for Subfield {
    type Err = PicaParseError;

    /// Parse a Pica+ subfield.
    ///
    /// # Example
    /// ```
    /// use pica::Subfield;
    ///
    /// let subfield = "\u{1f}a123".parse::<Subfield>().unwrap();
    /// assert_eq!(subfield.code(), 'a');
    /// assert_eq!(subfield.value(), "123");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_subfield(s) {
            Ok((_, subfield)) => Ok(subfield),
            _ => Err(PicaParseError {}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let subfield = Subfield::new('a', "abc");
        assert_eq!(subfield.code(), 'a');
        assert_eq!(subfield.value(), "abc");
    }

    #[test]
    fn test_fmt() {
        let subfield = Subfield::new('a', "foo");
        assert_eq!(format!("{}", subfield), "$a foo");
        assert_eq!(subfield.to_string(), "$a foo");
    }

    #[test]
    fn test_from_str() {
        let subfield = "\u{1f}a123".parse::<Subfield>().unwrap();
        assert_eq!(subfield.code, 'a');
        assert_eq!(subfield.value, "123");

        assert!("\u{1f}".parse::<Subfield>().is_err());
    }
}
