//! This module provides a data structure and functions related to a PICA+
//! record.

use crate::error::ParsePicaError;
use crate::parser::parse_record;
use crate::select::{Range, Selector, Selectors};
use crate::{Field, Path};

use nom::{combinator::all_consuming, Finish};

use serde::Serialize;
use std::borrow::Cow;
use std::ops::Deref;

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct Record<'a>(Vec<Field<'a>>);

impl<'a> Record<'a> {
    /// Creates a new record
    ///
    /// # Arguments
    ///
    /// * A vector of [`Field`]s
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Record, Subfield};
    ///
    /// let record = Record::new(vec![Field::new("003@", None, vec![])]);
    /// assert_eq!(record.len(), 1);
    /// ```
    pub fn new(fields: Vec<Field<'a>>) -> Self {
        Self(fields)
    }

    /// Decodes a record
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice holding a PICA+ record
    ///
    /// # Example
    /// ```
    /// use pica::Record;
    ///
    /// let result = Record::decode("003@ \u{1f}0123456789X\u{1e}");
    /// assert!(result.is_ok());
    /// ```
    pub fn decode(input: &'a str) -> Result<Self, ParsePicaError> {
        match all_consuming(parse_record)(input).finish() {
            Ok((_remaining, record)) => Ok(record),
            _ => Err(ParsePicaError::InvalidRecord),
        }
    }

    /// Returns the record as an pretty formatted string.
    ///
    /// # Example
    /// ```
    /// use pica::{Field, Record, Subfield};
    ///
    /// let record = Record::new(vec![
    ///     Field::new(
    ///         "003@",
    ///         None,
    ///         vec![Subfield::new('0', "123456789X").unwrap()],
    ///     ),
    ///     Field::new(
    ///         "012A",
    ///         None,
    ///         vec![
    ///             Subfield::new('a', "123").unwrap(),
    ///             Subfield::new('b', "456").unwrap(),
    ///         ],
    ///     ),
    /// ]);
    /// assert_eq!(record.pretty(), "003@ $0 123456789X\n012A $a 123 $b 456");
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
                && field.occurrence().as_deref() == path.occurrence()
            {
                for subfield in &field.subfields {
                    if subfield.name() == path.name() {
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

    fn collect(&self, selector: &Selector) -> Vec<Vec<String>> {
        let mut retval: Vec<Vec<String>> = Vec::new();

        for field in self.iter() {
            if field.tag == selector.tag
                && selector.occurrence == field.occurrence()
            {
                let mut temp = vec![];
                for (name, range) in &selector.subfields {
                    let mut values: Vec<Cow<'_, str>> = field
                        .subfields()
                        .iter()
                        .filter(|subfield| subfield.name == *name)
                        .map(|subfield| subfield.value.clone())
                        .collect::<Vec<_>>();

                    let values_ranged = if let Some(range) = range {
                        match range {
                            Range::Range(start, end) => &values[*start..*end],
                            Range::RangeTo(end) => &values[..*end],
                            Range::RangeFrom(start) => &values[*start..],
                            Range::RangeFull => &values[..],
                        }
                    } else {
                        &values[..]
                    };

                    values = values_ranged.to_vec();

                    if values.is_empty() {
                        values.push(Cow::Borrowed(""))
                    }

                    temp.push(values);
                }

                let mut result = temp.iter().fold(vec![vec![]], |acc, x| {
                    let mut tmp: Vec<Vec<String>> = vec![];

                    for item in x {
                        for row in &acc {
                            let mut new_row: Vec<String> = row.clone();
                            new_row.push(String::from(item.clone()));
                            tmp.push(new_row);
                        }
                    }

                    tmp
                });

                retval.append(&mut result);
            }
        }

        if retval.is_empty() {
            retval.push(
                selector
                    .subfields
                    .iter()
                    .map(|_| "".to_string())
                    .collect::<Vec<_>>(),
            )
        }

        retval
    }

    pub fn select(&self, selectors: &Selectors) -> Vec<Vec<String>> {
        let result = selectors
            .iter()
            .map(|selector| self.collect(&selector))
            .fold(vec![vec![]], |acc, mut x| {
                if x.is_empty() {
                    x = vec![vec!["".to_string()]];
                }

                let mut tmp: Vec<Vec<String>> = vec![];
                for item in x {
                    for row in &acc {
                        let mut new_row: Vec<String> = row.clone();
                        new_row.append(&mut item.clone());
                        tmp.push(new_row.clone());
                    }
                }

                tmp
            });

        result
    }
}

impl<'a> Deref for Record<'a> {
    type Target = Vec<Field<'a>>;

    fn deref(&self) -> &Vec<Field<'a>> {
        &self.0
    }
}
