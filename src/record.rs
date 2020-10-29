use crate::{error::PicaParseError, parser::parse_record, Field};
use std::{default::Default, fmt, str::FromStr};

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

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.fields
                .iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Subfield;

    #[test]
    fn test_from_str() {
        let record = Record::from_str("003@ \u{1f}0123456789\u{1e}").unwrap();
        assert_eq!(record.fields.len(), 1);

        assert!(Record::from_str("003@ \u{1f}0123456789").is_err());
    }

    #[test]
    fn test_fmt() {
        let record = Record::new();
        assert_eq!(record.to_string(), "");

        let mut record = Record::new();
        record.fields.push(Field::new(
            "012A",
            None,
            vec![Subfield::new('a', "123"), Subfield::new('b', "456")],
        ));
        record.fields.push(Field::new(
            "012@",
            Some("00"),
            vec![Subfield::new('c', "567")],
        ));

        assert_eq!(record.to_string(), "012A $a 123 $b 456\n012@/00 $c 567");
    }
}
