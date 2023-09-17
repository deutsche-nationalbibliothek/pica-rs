use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::io::{self, Cursor, Write};
use std::ops::{Deref, DerefMut};
use std::slice::Iter;
use std::str::Utf8Error;

use bstr::{BStr, BString};
use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::multi::many1;
use nom::sequence::terminated;
use nom::Finish;
use sha2::{Digest, Sha256};

use crate::field::{parse_field, RawField};
use crate::parser::{ParseResult, LF};
use crate::{Field, FieldRef, ParsePicaError};

/// A PICA+ record.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Record<T: AsRef<[u8]>>(Vec<Field<T>>);

/// A immutable PICA+ record.
pub type RecordRef<'a> = Record<&'a BStr>;

/// A mutable PICA+ tag.
pub type RecordMut = Record<BString>;

/// A PICA+ record, that may contain invalid UTF-8 data.
#[derive(Debug)]
pub struct ByteRecord<'a> {
    raw_data: Option<&'a [u8]>,
    record: RecordRef<'a>,
}

/// A PICA+ record, that guarantees valid UTF-8 data.
#[derive(Debug)]
pub struct StringRecord<'a>(ByteRecord<'a>);

impl<T: AsRef<[u8]>> Record<T> {
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

    /// Retains only the fields specified by the predicate.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{RecordRef, TagRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut record = RecordRef::new(vec![
    ///         ("003@", None, vec![('0', "123456789X")]),
    ///         ("002@", None, vec![('0', "Oaf")]),
    ///     ]);
    ///
    ///     record.retain(|field| field.tag() == &TagRef::new("003@"));
    ///     assert_eq!(record.iter().len(), 1);
    ///     Ok(())
    /// }
    /// ```
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Field<T>) -> bool,
    {
        self.0.retain(f);
    }
}

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

    /// Write the record into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let record = RecordRef::from_bytes(b"003@ \x1f0a\x1e\n")?;
    ///     record.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #     String::from_utf8(writer.into_inner())?,
    ///     #     "003@ \x1f0a\x1e\n"
    ///     # );
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        if !self.is_empty() {
            for field in self.iter() {
                field.write_to(out)?;
            }
            writeln!(out)?;
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
        Ok(Self {
            record: RecordRef::from_bytes(data)?,
            raw_data: Some(data),
        })
    }

    /// Write the record into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::ByteRecord;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let record = ByteRecord::from_bytes(b"003@ \x1f0a\x1e\n")?;
    ///     record.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #     String::from_utf8(writer.into_inner())?,
    ///     #     "003@ \x1f0a\x1e\n"
    ///     # );
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        match self.raw_data {
            Some(data) => out.write_all(data),
            None => self.record.write_to(out),
        }
    }

    /// Retains only the fields specified by the predicate.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{ByteRecord, TagRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut record = ByteRecord::from_bytes(
    ///         b"003@ \x1f0a\x1e002@ \x1f0Olfo\x1e\n",
    ///     )?;
    ///
    ///     record.retain(|field| field.tag() == &TagRef::new("003@"));
    ///     assert_eq!(record.iter().len(), 1);
    ///     Ok(())
    /// }
    /// ```
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&FieldRef) -> bool,
    {
        self.record.retain(f);
        self.raw_data = None;
    }

    /// Returns the SHA-256 hash of the record.
    pub fn sha256(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let mut hasher = Sha256::new();

        let _ = self.write_to(&mut writer);
        let data = writer.into_inner();
        hasher.update(data);

        let result = hasher.finalize();
        result.to_vec()
    }

    pub fn into_inner(self) -> RecordRef<'a> {
        self.record
    }
}

impl<'a> Deref for ByteRecord<'a> {
    type Target = RecordRef<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.record
    }
}

impl<'a> DerefMut for ByteRecord<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.record
    }
}

impl<'a> From<RecordRef<'a>> for ByteRecord<'a> {
    fn from(record: RecordRef<'a>) -> Self {
        ByteRecord {
            raw_data: None,
            record,
        }
    }
}

impl<'a> Hash for ByteRecord<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.raw_data {
            Some(data) => data.hash(state),
            None => {
                let mut writer = Cursor::new(Vec::<u8>::new());
                let _ = self.write_to(&mut writer);
                let data = writer.into_inner();
                data.hash(state)
            }
        };
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
    use std::collections::hash_map::DefaultHasher;

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
    fn test_byte_record_hash() -> anyhow::Result<()> {
        let record =
            ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;
        let mut hasher = DefaultHasher::new();
        record.hash(&mut hasher);
        assert_eq!(hasher.finish(), 3101329223602639123);

        let record = ByteRecord::from(RecordRef::new(vec![(
            "003@",
            None,
            vec![('0', "123456789X")],
        )]));
        let mut hasher = DefaultHasher::new();
        record.hash(&mut hasher);
        assert_eq!(hasher.finish(), 3101329223602639123);

        Ok(())
    }

    #[test]
    fn test_byte_record_sha256() -> anyhow::Result<()> {
        let record =
            ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;

        assert_eq!(record.sha256(), b"K\x1f8\xbe\xf4m\xa5\xd0\x8b@{u7\x8bi\x96\x96\xc5\x91\xf6 \xddM\xd3\x8dy\xad[\x96;=\xb6");

        let record = ByteRecord::from(RecordRef::new(vec![(
            "003@",
            None,
            vec![('0', "123456789X")],
        )]));

        assert_eq!(record.sha256(), b"K\x1f8\xbe\xf4m\xa5\xd0\x8b@{u7\x8bi\x96\x96\xc5\x91\xf6 \xddM\xd3\x8dy\xad[\x96;=\xb6");

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
