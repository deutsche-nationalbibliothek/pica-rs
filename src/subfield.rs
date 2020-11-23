//! Pica+ Subfield

use crate::error::ParsePicaError;
use crate::parser::parse_subfield;
use nom::Finish;
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_subfield(s).finish() {
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
