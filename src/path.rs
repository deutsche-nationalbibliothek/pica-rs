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
use std::fmt;
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
        self.occurrence.as_deref()
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

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)?;

        if let Some(occurrence) = self.occurrence() {
            write!(f, "/{}", &occurrence)?;
        };

        write!(f, ".{}", self.code)?;

        if let Some(index) = self.index {
            write!(f, "[{}]", index)?;
        }

        Ok(())
    }
}
