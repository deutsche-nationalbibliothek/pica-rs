use crate::{error::PicaParseError, parser::parse_record, Field};
use std::{default::Default, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub struct Record {
    pub fields: Vec<Field>,
}

impl Record {
    /// Create a new record.
    ///
    /// # Example
    /// ```
    /// use pica::Record;
    ///
    /// let record = Record::new();
    /// assert!(record.fields.is_empty());
    /// ```
    pub fn new() -> Self {
        Record { fields: vec![] }
    }
}

impl Default for Record {
    /// Create an empty pica record.
    ///
    /// # Example
    /// ```
    /// use pica::Record;
    ///
    /// let record = Record::default();
    /// assert!(record.fields.is_empty());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for Record {
    type Err = PicaParseError;

    /// Parse a Pica+ record from a string slice.
    ///
    /// # Example
    /// ```
    /// use pica::Record;
    /// use std::str::FromStr;
    ///
    /// let result = Record::from_str("003@ \u{1f}0123456789\u{1e}");
    /// assert!(result.is_ok());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_record(s) {
            Ok((_, record)) => Ok(record),
            _ => Err(PicaParseError {}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let record = Record::from_str("003@ \u{1f}0123456789\u{1e}").unwrap();
        assert_eq!(record.fields.len(), 1);

        assert!(Record::from_str("003@ \u{1f}0123456789").is_err());
    }
}
