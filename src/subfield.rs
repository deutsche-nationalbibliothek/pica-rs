//! This module provides a data type and parser functions releated to a PICA+
//! subfield.

use crate::error::ParsePicaError;
use crate::parser::parse_subfield;

use nom::combinator::all_consuming;
use nom::Finish;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Subfield<'a> {
    pub(crate) name: char,
    pub(crate) value: Cow<'a, str>,
}

impl<'a> Subfield<'a> {
    /// Crates a new subfield
    ///
    /// # Arguments
    ///
    /// * `name` - An alpha-numeric ([0-9A-Za-z]) subfield code.
    /// * `value` - A string or string slice holding the subfield value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('0', "123456789X");
    /// assert!(subfield.is_ok());
    /// ```
    pub fn new<S>(name: char, value: S) -> Result<Self, ParsePicaError>
    where
        S: Into<Cow<'a, str>>,
    {
        let value = value.into();

        if !name.is_ascii_alphanumeric()
            || value.contains(&['\u{1e}', '\u{1f}'][..])
        {
            Err(ParsePicaError::InvalidSubfield)
        } else {
            Ok(Subfield { name, value })
        }
    }

    /// Decodes a subfield
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice holding a PICA+ encoded subfield
    ///
    /// # Example
    /// ```rust
    /// use pica::Subfield;
    ///
    /// let result = Subfield::decode("\u{1f}0123456789X");
    /// assert!(result.is_ok());
    /// ```
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_subfield)(input).finish() {
            Ok((_, subfield)) => Ok(subfield),
            _ => Err(ParsePicaError::InvalidSubfield),
        }
    }

    /// Encodes a subfield
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('0', "123456789X").expect("valid subfield");
    /// let encoded = subfield.encode();
    /// assert_eq!(encoded, "\u{1f}0123456789X");
    /// ```
    pub fn encode(&self) -> String {
        format!("\u{1f}{}{}", self.name, self.value)
    }

    /// Returns the name of the subfield.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('0', "123456789X").expect("valid subfield");
    /// assert_eq!(subfield.name(), '0');
    /// ```
    pub fn name(&self) -> char {
        self.name
    }

    /// Returns the value of the subfield
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('0', "123456789X").expect("valid subfield");
    /// assert_eq!(subfield.value(), "123456789X");
    /// ```
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }

    /// Returns the subfield as an human readable string
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// let subfield = Subfield::new('0', "123456789X").expect("valid subfield");
    /// assert_eq!(subfield.pretty(), "$0 123456789X");
    /// ```
    pub fn pretty(&self) -> String {
        format!("${} {}", self.name, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subfield_new() {
        // valid subfield
        let subfield = Subfield::new('0', "123456789X").unwrap();
        assert_eq!(subfield.name(), '0');
        assert_eq!(subfield.value(), "123456789X");

        // invalid subfield name
        assert!(Subfield::new('!', "123456789X").is_err());

        // invalid subfield value
        assert!(Subfield::new('a', "1\u{1e}23456789X").is_err());
        assert!(Subfield::new('a', "1\u{1f}23456789X").is_err());
    }

    #[test]
    fn test_subfield_decode() {
        assert!(Subfield::decode("\u{1f}0123456789X").is_ok());
        assert!(Subfield::decode("\u{1f}0").is_ok());
        assert!(Subfield::decode("\u{1f}!123456789X").is_err());
        assert!(Subfield::decode("\u{1e}0123456789X").is_err());
        assert!(Subfield::decode("\u{1f}0\u{1f}").is_err());
        assert!(Subfield::decode("").is_err());
    }

    #[test]
    fn test_encode() {
        assert_eq!(
            Subfield::new('0', "123456789X").unwrap().encode(),
            "\u{1f}0123456789X"
        );
    }
}
