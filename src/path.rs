//! Pica+ Path
//!
//! A path is a query syntax to address values within a pica+ record. The path
//! consists of a [`Field`] tag and a [`Subfield`] code. A [`Field`] occurrence
//! or an index is optional
//!
//! # Grammar
//!
//! ```text
//! path       ::= tag occurrence? code index?
//! tag        ::= [012] [0-9]{2} ([A-Z] | '@')
//! occurrence ::= '/' [0-9]{2,3}
//! code       ::= [a-z] | [A-Z] | [0-9]
//! index      ::= '[' [0-9]+ ']'
//! ```

use crate::error::ParsePicaError;
use crate::parser::parse_path;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    tag: String,
    occurrence: Option<String>,
    code: char,
    index: Option<usize>,
}

impl Path {
    pub fn new<S>(
        tag: S,
        occurrence: Option<S>,
        code: char,
        index: Option<usize>,
    ) -> Self
    where
        S: Into<String>,
    {
        Path {
            tag: tag.into(),
            occurrence: occurrence.map(|o| o.into()),
            code,
            index,
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn occurrence(&self) -> Option<&str> {
        self.occurrence.as_ref().map(|s| s.as_str())
    }

    pub fn code(&self) -> char {
        self.code
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }
}

impl FromStr for Path {
    type Err = ParsePicaError;

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
    fn test_parse_path() {
        let data: Vec<(&str, Path)> = vec![
            ("003@.0", Path::new("003@", None, '0', None)),
            ("  003@.0 ", Path::new("003@", None, '0', None)),
            ("012A/01.0", Path::new("012A", Some("01"), '0', None)),
            ("003@.0[0]", Path::new("003@", None, '0', Some(0))),
            ("012A/00.0[1]", Path::new("012A", Some("00"), '0', Some(1))),
        ];

        for (input, expected) in data {
            assert_eq!(parse_path(input), Ok(("", expected)));
        }
    }

    #[test]
    fn test_path_new() {
        let path = Path::new("012A", Some("000"), '0', None);
        assert_eq!(path.tag(), "012A");
        assert_eq!(path.occurrence(), Some("000"));
        assert_eq!(path.code(), '0');
    }

    #[test]
    fn test_path_from_str() {
        let path = "012A/000.0".parse::<Path>().unwrap();
        assert_eq!(path.tag(), "012A");
        assert_eq!(path.occurrence(), Some("000"));
        assert_eq!(path.code(), '0');
        assert_eq!(path.index(), None);

        let result = "003@.?".parse::<Path>();
        assert_eq!(result.err(), Some(ParsePicaError::InvalidPath));
    }
}
