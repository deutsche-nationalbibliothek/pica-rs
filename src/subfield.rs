//! This module provides a data type and parser functions releated to a PICA+
//! subfield.
//!
//! # Grammar
//!
//! ```text
//! Subfield := '\x1f' Code Value
//! Code     := [0-9A-Za-z]
//! Value    := [^\x1e\x1d]*
//! ```

use crate::error::ParsePicaError;

use nom::character::complete::{char, none_of, satisfy};
use nom::combinator::{all_consuming, map, recognize};
use nom::multi::many0;
use nom::sequence::{pair, preceded};
use nom::{Finish, IResult};

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

/// Parses a subield name
pub(crate) fn parse_subfield_name(i: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_alphanumeric())(i)
}

/// Parses a subfield value
fn parse_subfield_value(i: &str) -> IResult<&str, &str> {
    recognize(many0(none_of("\u{1e}\u{1f}")))(i)
}

// Parses a subfield
pub(crate) fn parse_subfield(i: &str) -> IResult<&str, Subfield> {
    preceded(
        char('\u{1f}'),
        map(
            pair(parse_subfield_name, parse_subfield_value),
            |(name, value)| Subfield {
                name,
                value: value.into(),
            },
        ),
    )(i)
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

    #[test]
    fn test_parse_subfield_name() {
        for range in vec!['a'..='z', 'A'..='Z', '0'..='9'] {
            for c in range {
                assert_eq!(parse_subfield_name(&String::from(c)), Ok(("", c)));
            }
        }
    }

    #[test]
    fn test_parse_subfield_value() {
        assert_eq!(parse_subfield_value(""), Ok(("", "")));
        assert_eq!(parse_subfield_value("abc"), Ok(("", "abc")));
        assert_eq!(parse_subfield_value("ab\u{1f}c"), Ok(("\u{1f}c", "ab")));
        assert_eq!(parse_subfield_value("ab\u{1e}c"), Ok(("\u{1e}c", "ab")));
    }

    #[test]
    fn test_parse_subfield() {
        assert_eq!(
            parse_subfield("\u{1f}a123"),
            Ok(("", Subfield::new('a', "123").unwrap()))
        );
        assert!(parse_subfield("!a123").is_err());
    }
}
