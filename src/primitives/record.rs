use std::io::{self, Write};
use std::str::Utf8Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use winnow::Parser;

use super::parse::parse_record_ref;
use super::{Field, FieldRef, ParsePicaError};

/// An immutable PICA+ record.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RecordRef<'a>(
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(super)  Vec<FieldRef<'a>>,
);

impl<'a> RecordRef<'a> {
    /// Creates a new [RecordRef].
    ///
    /// # Errors
    ///
    /// This function fails if either the tag, occcurrence or any
    /// subfield is nvalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::RecordRef;
    ///
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// assert_eq!(record.fields().len(), 2);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<T>(
        fields: Vec<(&'a str, Option<&'a str>, T)>,
    ) -> Result<Self, ParsePicaError>
    where
        T: IntoIterator<Item = (char, &'a str)>,
    {
        let fields = fields
            .into_iter()
            .map(|(tag, occ, subfields)| {
                FieldRef::new(tag, occ, subfields)
            })
            .collect::<Result<Vec<FieldRef<'a>>, _>>()?;

        Ok(Self(fields))
    }

    /// Creates a new [RecordRef] from a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::RecordRef;
    ///
    /// let record = RecordRef::from_bytes(b"012A \x1f0abc\x1e\n")?;
    /// assert_eq!(record.fields().len(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes<B>(record: &'a B) -> Result<Self, ParsePicaError>
    where
        B: AsRef<[u8]> + ?Sized,
    {
        let bytes = record.as_ref();

        parse_record_ref.parse(bytes).map_err(|_| {
            ParsePicaError(format!("invalid record: '{bytes:?}'"))
        })
    }

    /// Returns the fields of the record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{FieldRef, RecordRef};
    ///
    /// let record = RecordRef::from_bytes(b"012A \x1f0abc\x1e\n")?;
    /// let field = FieldRef::from_bytes(b"012A \x1f0abc\x1e")?;
    /// assert_eq!(record.fields(), [field]);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn fields(&self) -> &[FieldRef<'a>] {
        &self.0
    }

    /// Returns `true` if the [RecordRef] contains no fields, otherwise
    /// `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::RecordRef;
    ///
    /// let record = RecordRef::from_bytes(b"002@ \x1f0Oaf\x1e\n")?;
    /// assert!(!record.is_empty());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Retains only the [FieldRef]s specified by the predicate.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::RecordRef;
    ///
    /// let mut record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Oaf")]),
    /// ])?;
    ///
    /// assert_eq!(record.fields().len(), 2);
    ///
    /// record.retain(|field| field.tag() == "003@");
    /// assert_eq!(record.fields().len(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn retain<F: FnMut(&FieldRef) -> bool>(&mut self, f: F) {
        self.0.retain(f);
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the record
    /// contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::RecordRef;
    ///
    /// let record = RecordRef::from_bytes(b"003@ \x1f0a\x1e\n")?;
    /// assert!(record.validate().is_ok());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn validate(&self) -> Result<(), Utf8Error> {
        for field in self.fields() {
            field.validate()?;
        }

        Ok(())
    }

    /// Write the [RecordRef] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::primitives::RecordRef;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let record = RecordRef::from_bytes(b"003@ \x1f0a\x1e\n")?;
    /// record.write_to(&mut writer);
    ///
    /// assert_eq!(
    ///     String::from_utf8(writer.into_inner())?,
    ///     "003@ \x1f0a\x1e\n"
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        if !self.is_empty() {
            for field in self.fields() {
                field.write_to(out)?;
            }

            writeln!(out)?;
        }

        Ok(())
    }
}

/// A mutable record.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Record(Vec<Field>);

impl Record {
    /// Write the record into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::primitives::{Record, RecordRef};
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let record: Record =
    ///     RecordRef::from_bytes(b"003@ \x1f0a\x1e\n")?.into();
    /// record.write_to(&mut writer);
    ///
    /// assert_eq!(
    ///     String::from_utf8(writer.into_inner())?,
    ///     "003@ \x1f0a\x1e\n"
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        if !self.0.is_empty() {
            for field in self.0.iter() {
                field.write_to(out)?;
            }

            writeln!(out)?;
        }

        Ok(())
    }
}

impl From<RecordRef<'_>> for Record {
    /// Converts a [RecordRef] into a [Record].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{Record, RecordRef};
    ///
    /// let record_ref = RecordRef::from_bytes(b"003@ \x1f0a\x1e\n")?;
    /// let record = Record::from(record_ref);
    /// assert_eq!(RecordRef::from_bytes(b"003@ \x1f0a\x1e\n")?, record);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn from(record: RecordRef<'_>) -> Self {
        Self(record.0.into_iter().map(Field::from).collect())
    }
}

impl PartialEq<Record> for RecordRef<'_> {
    fn eq(&self, other: &Record) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<RecordRef<'_>> for Record {
    fn eq(&self, other: &RecordRef<'_>) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Record {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let fields = (0..g.size())
            .map(|_| Field::arbitrary(g))
            .collect::<Vec<_>>();

        Self(fields)
    }
}
