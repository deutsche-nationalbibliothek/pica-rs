use std::fmt::Display;
use std::slice::Iter;

use bstr::{BStr, BString};
use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::multi::many1;
use nom::sequence::terminated;
use nom::Finish;

use crate::field::{parse_field, RawField};
use crate::parser::{ParseResult, LF};
use crate::{Field, ParsePicaError};

/// A PICA+ record.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Record<T>(pub(crate) Vec<Field<T>>);

/// A immutable PICA+ record.
pub type RecordRef<'a> = Record<&'a BStr>;

/// A mutable PICA+ tag.
pub type RecordMut = Record<BString>;

impl<'a, T: AsRef<[u8]> + From<&'a BStr> + Display> Record<T> {
    /// Create a new record.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record =
    ///         RecordRef::new(vec![("003@", None, vec![('0', "abc")])]);
    ///     assert_eq!(record.iter().len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn new<U: Into<T>>(
        fields: Vec<(U, Option<U>, Vec<(char, U)>)>,
    ) -> Self {
        let fields = fields
            .into_iter()
            .map(|(tag, occurrence, subfields)| {
                Field::new(tag, occurrence, subfields)
            })
            .collect();

        Self(fields)
    }

    /// Creates an PICA+ record from a byte slice.
    ///
    /// If an invalid record is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record = RecordRef::from_bytes(b"003@ \x1f0abc\x1e\n");
    ///     assert_eq!(record.iter().len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_record(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidRecord)
            .map(|(_, fields)| {
                Self(
                    fields
                        .into_iter()
                        .map(|(t, o, s)| Field::new(t, o, s))
                        .collect(),
                )
            })
    }

    /// Returns an iterator over the fields of the record.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record = RecordRef::new(vec![
    ///         ("003@", None, vec![('0', "123456789X")]),
    ///         ("002@", None, vec![('0', "Oaf")]),
    ///     ]);
    ///
    ///     assert_eq!(record.iter().len(), 2);
    ///     Ok(())
    /// }
    /// ```
    pub fn iter(&self) -> Iter<Field<T>> {
        self.0.iter()
    }

    /// Returns `true` if the record contains no fields, otherwise
    /// `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record =
    ///         RecordRef::new(vec![("002@", None, vec![('0', "Oaf")])]);
    ///     assert!(!record.is_empty());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

#[inline]
pub fn parse_record(i: &[u8]) -> ParseResult<Vec<RawField>> {
    all_consuming(terminated(many1(parse_field), char(LF as char)))(i)
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_parse_field_value() {
        assert_done!(parse_record(b"003@ \x1f0123456789X\x1e\n"));
        assert_done!(parse_record(
            b"003@ \x1f0123456789X\x1e002@ \x1fOaf\x1e\n"
        ));

        assert_error!(parse_record(b"003@ \x1f0123456789X\x1e"));
        assert_error!(parse_record(b"\n"));
    }
}
