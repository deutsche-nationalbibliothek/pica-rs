use crate::error::ParsePicaError;
use crate::filter::{BooleanOp, ComparisonOp};
use crate::parser::parse_record;
use crate::{Field, Filter, Path};
use regex::Regex;
use serde::Serialize;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct Record(Vec<Field>);

impl Record {
    pub fn new(fields: Vec<Field>) -> Self {
        Record(fields)
    }

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

        if let Some(index) = path.index() {
            if let Some(value) = result.get(index) {
                return vec![value];
            } else {
                return vec![];
            }
        }

        result
    }

    pub fn matches(&self, filter: &Filter) -> bool {
        match filter {
            Filter::ExistenceExpr(path) => !self.path(path).is_empty(),
            Filter::GroupedExpr(filter) => self.matches(filter),
            Filter::BooleanExpr(lhs, op, rhs) => match op {
                BooleanOp::And => self.matches(lhs) && self.matches(rhs),
                BooleanOp::Or => self.matches(lhs) || self.matches(rhs),
            },
            Filter::ComparisonExpr(path, op, rvalue) => {
                let lvalues = self.path(path);
                match op {
                    ComparisonOp::Eq => {
                        lvalues.into_iter().any(|x| x == rvalue)
                    }
                    ComparisonOp::Ne => {
                        lvalues.into_iter().all(|x| x != rvalue)
                    }
                    ComparisonOp::Re => {
                        let re = Regex::new(rvalue).unwrap();
                        lvalues.into_iter().any(|x| re.is_match(x))
                    }
                    ComparisonOp::StartsWith => {
                        lvalues.into_iter().any(|x| x.starts_with(rvalue))
                    }
                }
            }
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
