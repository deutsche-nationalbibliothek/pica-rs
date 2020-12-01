//! Pica+ Subfield

use crate::error::ParsePicaError;
use crate::parser::parse_subfield;
use nom::Finish;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Subfield<'a> {
    pub(crate) code: char,
    pub(crate) value: Cow<'a, str>,
}

impl<'a> Subfield<'a> {
    pub fn new<S>(code: char, value: S) -> Result<Self, ParsePicaError>
    where
        S: Into<Cow<'a, str>>,
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
        S: Into<Cow<'a, str>>,
    {
        Self {
            code,
            value: value.into(),
        }
    }

    pub fn decode(s: &'a str) -> Result<Self, ParsePicaError> {
        match parse_subfield(s).finish() {
            Ok((_, subfield)) => Ok(subfield),
            _ => Err(ParsePicaError::InvalidSubfield),
        }
    }

    /// Returns the code of the subfield.
    pub fn code(&self) -> char {
        self.code
    }

    // Returns the value of the subfield.
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }

    /// Returns the subfield as an PICA3 encoded string.
    pub fn pretty(&self) -> String {
        format!("${} {}", self.code, self.value)
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
