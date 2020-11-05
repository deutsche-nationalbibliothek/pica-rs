use crate::error::ParsePicaError;
use crate::parser::parse_path;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Path {
    tag: String,
    occurrence: String,
    code: char,
}

impl Path {
    /// Creates a new path.
    ///
    /// # Example
    /// ```
    /// use pica::Path;
    ///
    /// let path = Path::new("012A", "000", '0');
    /// assert_eq!(path.tag(), "012A");
    /// assert_eq!(path.occurrence(), "000");
    /// assert_eq!(path.code(), '0');
    /// ```
    pub fn new<S>(tag: S, occurrence: S, code: char) -> Self
    where
        S: Into<String>,
    {
        Path {
            tag: tag.into(),
            occurrence: occurrence.into(),
            code,
        }
    }

    /// Returns the tag of the path.
    ///
    /// # Example
    /// ```
    /// use pica::Path;
    ///
    /// let path = Path::new("012A", "000", '0');
    /// assert_eq!(path.tag(), "012A");
    /// ```
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the occurrence of the path.
    ///
    /// # Example
    /// ```
    /// use pica::Path;
    ///
    /// let path = Path::new("012A", "000", '0');
    /// assert_eq!(path.occurrence(), "000");
    /// ```
    pub fn occurrence(&self) -> &str {
        &self.occurrence
    }

    /// Returns the code of the path.
    ///
    /// # Example
    /// ```
    /// use pica::Path;
    ///
    /// let path = Path::new("012A", "000", '0');
    /// assert_eq!(path.code(), '0');
    /// ```
    pub fn code(&self) -> char {
        self.code
    }
}

impl FromStr for Path {
    type Err = ParsePicaError;

    /// Parse a pica+ path.
    ///
    /// A Pica+ path is a tag, an optional occurrence and an optional code.
    ///
    /// # Grammar
    ///
    /// A path which conform to the following [EBNF] grammar will result in an
    /// [`Ok`] being returned.
    ///
    /// ```text
    /// Path      ::= Tag Occurrence? Code?
    /// Tag        ::= [012] [0-9]{2} ([A-Z] | '@')
    /// Occurrence ::= '/' [0-9]{2,3}
    /// Code       ::= [a-zA-Z0-9]
    /// ```
    ///
    /// [EBNF]: https://www.w3.org/TR/REC-xml/#sec-notation
    ///
    /// # Example
    /// ```
    /// use pica::Path;
    ///
    /// let path = "003@.0".parse::<Path>().unwrap();
    /// assert_eq!(path.tag(), "003@");
    /// assert_eq!(path.occurrence(), "");
    /// assert_eq!(path.code(), '0');
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_path(s) {
            Ok((_, path)) => Ok(path),
            _ => Err(ParsePicaError::InvalidPath),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_new() {
        let path = Path::new("012A", "000", '0');
        assert_eq!(path.tag(), "012A");
        assert_eq!(path.occurrence(), "000");
        assert_eq!(path.code(), '0');
    }

    #[test]
    fn test_path_from_str() {
        let path = "012A/000.0".parse::<Path>().unwrap();
        assert_eq!(path.tag(), "012A");
        assert_eq!(path.occurrence(), "000");
        assert_eq!(path.code(), '0');

        let result = "003@.?".parse::<Path>();
        assert_eq!(result.err(), Some(ParsePicaError::InvalidPath));
    }
}
