use std::fmt::Display;
use std::ops::Deref;
use std::slice::Iter;
use std::str::Utf8Error;

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
pub struct Record<T>(Vec<Field<T>>);

/// A immutable PICA+ record.
pub type RecordRef<'a> = Record<&'a BStr>;

/// A mutable PICA+ tag.
pub type RecordMut = Record<BString>;

/// A PICA+ record, that may contian invalid UTF-8 data.
pub struct ByteRecord<'a>(RecordRef<'a>);

/// A PICA+ record, that guarantees valid UTF-8 data.
pub struct StringRecord<'a>(ByteRecord<'a>);

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
            .map_err(|_| ParsePicaError::InvalidRecord(data.into()))
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

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the record
    /// contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record = RecordRef::from_bytes(b"003@ \x1f0a\x1e\n")?;
    ///     assert!(record.validate().is_ok());
    ///
    ///     let record =
    ///         RecordRef::from_bytes(b"003@ \x1f0\x00\x9F\x1e\n")?;
    ///     assert!(record.validate().is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), Utf8Error> {
        for field in self.iter() {
            field.validate()?;
        }

        Ok(())
    }
}

#[inline]
fn parse_record(i: &[u8]) -> ParseResult<Vec<RawField>> {
    all_consuming(terminated(many1(parse_field), char(LF as char)))(i)
}

impl<'a> ByteRecord<'a> {
    /// Creates an PICA+ record from a byte slice.
    ///
    /// If an invalid record is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::ByteRecord;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record = ByteRecord::from_bytes(b"003@ \x1f0abc\x1e\n");
    ///     assert_eq!(record.iter().len(), 1);
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParsePicaError> {
        Ok(Self(RecordRef::from_bytes(data)?))
    }
}

impl<'a> Deref for ByteRecord<'a> {
    type Target = RecordRef<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> StringRecord<'a> {
    /// Creates an PICA+ record from a byte slice.
    ///
    /// If an invalid record is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::StringRecord;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record = StringRecord::from_bytes(b"003@ \x1f0abc\x1e\n")?;
    ///     assert_eq!(record.iter().len(), 1);
    ///
    ///     let result =
    ///         StringRecord::from_bytes(b"003@ \x1f0\x00\x9f\x1e\n");
    ///     assert!(result.is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParsePicaError> {
        Self::try_from(ByteRecord::from_bytes(data)?)
            .map_err(|_| ParsePicaError::InvalidRecord(data.into()))
    }
}

impl<'a> Deref for StringRecord<'a> {
    type Target = ByteRecord<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<ByteRecord<'a>> for StringRecord<'a> {
    type Error = Utf8Error;

    fn try_from(record: ByteRecord<'a>) -> Result<Self, Self::Error> {
        record.validate()?;

        Ok(StringRecord(record))
    }
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_byte_record() -> anyhow::Result<()> {
        let record =
            ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;

        assert!(record.validate().is_ok());
        assert!(!record.is_empty());

        Ok(())
    }

    #[test]
    fn test_string_record() -> anyhow::Result<()> {
        let record =
            StringRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;

        assert!(record.validate().is_ok());
        assert!(!record.is_empty());

        let record =
            ByteRecord::from_bytes(b"003@ \x1f0\x00\x9f\x1e\n")?;
        assert!(StringRecord::try_from(record).is_err());

        Ok(())
    }

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
