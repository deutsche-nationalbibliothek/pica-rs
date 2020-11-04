use crate::error::ParsePicaError;
use crate::parser::parse_record;
use crate::{ComparisonOp, Expr, Field, LogicalOp, Path};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Record(Vec<Field>);

impl Record {
    /// Creates a new record.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Record, Subfield};
    ///
    /// let record = Record::new(vec![Field::new(
    ///     "003@",
    ///     "",
    ///     vec![Subfield::new('0', "123").unwrap()],
    /// )]);
    /// assert_eq!(record.len(), 1);
    /// ```
    pub fn new(fields: Vec<Field>) -> Self {
        Record(fields)
    }

    /// Returns the field as an pretty formatted string.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Record, Subfield};
    ///
    /// let record = Record::new(vec![
    ///     Field::new("003@", "", vec![Subfield::new('0', "123456789").unwrap()]),
    ///     Field::new(
    ///         "012A",
    ///         "00",
    ///         vec![
    ///             Subfield::new('a', "123").unwrap(),
    ///             Subfield::new('b', "456").unwrap(),
    ///         ],
    ///     ),
    /// ]);
    /// assert_eq!(record.pretty(), "003@ $0 123456789\n012A/00 $a 123 $b 456");
    /// ```
    pub fn pretty(&self) -> String {
        String::from(
            &*self
                .iter()
                .map(|s| s.pretty())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    pub fn path(&self, path: &Path) -> Vec<&str> {
        let mut result: Vec<&str> = Vec::new();

        for field in &self.0 {
            if field.tag() == path.tag()
                && field.occurrence() == path.occurrence()
            {
                for subfield in &field.subfields {
                    if subfield.code() == path.code() {
                        result.push(subfield.value());
                    }
                }
            }
        }

        result
    }

    pub fn matches(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Predicate(path, op, rvalue) => {
                let lvalues = self.path(path);
                match op {
                    ComparisonOp::Eq => {
                        lvalues.into_iter().any(|x| x == rvalue)
                    }
                    ComparisonOp::Ne => {
                        lvalues.into_iter().any(|x| x != rvalue)
                    }
                }
            }
            Expr::Connective(lhs, op, rhs) => match op {
                LogicalOp::And => self.matches(lhs) && self.matches(rhs),
                LogicalOp::Or => self.matches(lhs) || self.matches(rhs),
            },
        }
    }
}

impl FromStr for Record {
    type Err = ParsePicaError;

    /// Parse a pica+ encoded record.
    ///
    /// A Pica+ record is just a list of [`Field`].
    ///
    /// # Grammar
    ///
    /// A record which conform to the following [EBNF] grammar will result in
    /// an [`Ok`] being returned.
    ///
    /// ```text
    /// Record     ::= Field*
    /// Field      ::= Tag Occurrence? Subfield* '#x1e'
    /// Tag        ::= [012] [0-9]{2} ([A-Z] | '@')
    /// Occurrence ::= '/' [0-9]{2,3}
    /// Subfield   ::= Code Value
    /// Code       ::= [a-zA-Z0-9]
    /// Value      ::= [^#x1e#x1f]
    /// ```
    ///
    /// [EBNF]: https://www.w3.org/TR/REC-xml/#sec-notation
    ///
    /// # Example
    /// ```
    /// use pica::Record;
    ///
    /// assert!("003@ \u{1f}0123456789\u{1e}".parse::<Record>().is_ok());
    /// assert!("\u{1f}!123456789".parse::<Record>().is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_record(s) {
            Ok((_, record)) => Ok(record),
            _ => Err(ParsePicaError::InvalidRecord),
        }
    }
}

impl Deref for Record {
    type Target = Vec<Field>;

    fn deref(&self) -> &Vec<Field> {
        &self.0
    }
}

impl DerefMut for Record {
    fn deref_mut(&mut self) -> &mut Vec<Field> {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Field, Subfield};

    #[test]
    fn test_record_new() {
        let field1 =
            Field::new("003@", "", vec![Subfield::new('0', "12345").unwrap()]);
        let field2 =
            Field::new("012A", "00", vec![Subfield::new('a', "abc").unwrap()]);

        let record = Record::new(vec![field1.clone(), field2.clone()]);
        assert!(record.contains(&field1));
        assert!(record.contains(&field2));
        assert_eq!(record.len(), 2);
    }

    #[test]
    fn test_record_from_str() {
        let record: Record = "003@ \u{1f}0123\u{1e}012A/00 \u{1f}a123\u{1e}"
            .parse()
            .unwrap();

        let field =
            Field::new("003@", "", vec![Subfield::new('0', "123").unwrap()]);
        assert!(record.contains(&field));

        let field =
            Field::new("012A", "00", vec![Subfield::new('a', "123").unwrap()]);
        assert!(record.contains(&field));

        assert_eq!(record.len(), 2);
    }
}
