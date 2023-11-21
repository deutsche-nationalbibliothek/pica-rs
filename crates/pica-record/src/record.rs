use std::hash::{Hash, Hasher};
use std::io::{self, Cursor, Write};
use std::ops::{Deref, DerefMut};
use std::slice::Iter;
use std::str::Utf8Error;

use sha2::{Digest, Sha256};
use winnow::combinator::{repeat, terminated};
use winnow::{PResult, Parser};

use crate::field::parse_field;
use crate::{Field, FieldRef, ParsePicaError};

/// An immutable PICA+ record.
#[derive(Debug)]
pub struct RecordRef<'a>(Vec<FieldRef<'a>>);

/// An immutable PICA+ record.
#[derive(Debug)]
pub struct Record(Vec<Field>);

#[inline]
fn parse_record<'a>(i: &mut &'a [u8]) -> PResult<RecordRef<'a>> {
    terminated(repeat(1.., parse_field), b'\n')
        .map(RecordRef)
        .parse_next(i)
}

impl<'a> RecordRef<'a> {
    /// Create a new immutable record.
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
    pub fn new<B>(
        fields: Vec<(&'a B, Option<&'a B>, Vec<(char, &'a B)>)>,
    ) -> Self
    where
        B: ?Sized + AsRef<[u8]>,
    {
        let fields = fields
            .into_iter()
            .map(|(t, o, s)| FieldRef::new(t, o, s))
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
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_record
            .parse(bytes)
            .map_err(|_| ParsePicaError::InvalidRecord(bytes.into()))
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
    ///     let record = RecordRef::from_bytes(b"002@ \x1f0Oaf\x1e\n")?;
    ///     assert!(!record.is_empty());
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
    pub fn iter(&self) -> Iter<FieldRef> {
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
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn retain<F: FnMut(&FieldRef) -> bool>(&mut self, f: F) {
        self.0.retain(f);
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
    ///
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

impl PartialEq<RecordRef<'_>> for RecordRef<'_> {
    fn eq(&self, other: &RecordRef<'_>) -> bool {
        self.0 == other.0
    }
}

impl From<RecordRef<'_>> for Record {
    fn from(other: RecordRef<'_>) -> Self {
        Self(other.0.into_iter().map(Field::from).collect())
    }
}

/// A PICA+ record, that may contain invalid UTF-8 data.
#[derive(Debug)]
pub struct ByteRecord<'a> {
    raw_data: Option<&'a [u8]>,
    record: RecordRef<'a>,
}

impl<'a> ByteRecord<'a> {
    /// Creates an byte record from a byte slice.
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
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParsePicaError> {
        Ok(Self {
            record: RecordRef::from_bytes(bytes)?,
            raw_data: Some(bytes),
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
    pub fn retain<F: FnMut(&FieldRef) -> bool>(&mut self, f: F) {
        self.record.retain(f);
        self.raw_data = None;
    }

    /// Returns the SHA-256 hash of the record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::ByteRecord;
    /// use std::fmt::Write;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut record =
    ///         ByteRecord::from_bytes(b"012A \x1fa123\x1e\n")?;
    ///
    ///     let hash = record.sha256().iter().fold(
    ///         String::new(), |mut out, b| {
    ///             let _ = write!(out, "{b:02x}");
    ///             out
    ///         });
    ///
    ///     assert!(hash.starts_with("95e266"));
    ///     Ok(())
    /// }
    pub fn sha256(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let mut hasher = Sha256::new();

        let _ = self.write_to(&mut writer);
        let data = writer.into_inner();
        hasher.update(data);

        let result = hasher.finalize();
        result.to_vec()
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

impl PartialEq<ByteRecord<'_>> for ByteRecord<'_> {
    fn eq(&self, other: &ByteRecord<'_>) -> bool {
        match (self.raw_data, other.raw_data) {
            (Some(lhs), Some(rhs)) => lhs == rhs,
            _ => self.record == other.record,
        }
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

/// A PICA+ record, that guarantees valid UTF-8 data.
#[derive(Debug)]
pub struct StringRecord<'a>(ByteRecord<'a>);

impl<'a> TryFrom<ByteRecord<'a>> for StringRecord<'a> {
    type Error = Utf8Error;

    fn try_from(record: ByteRecord<'a>) -> Result<Self, Self::Error> {
        record.validate()?;

        Ok(StringRecord(record))
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
    ///     let record = StringRecord::from_bytes(b"003@ \x1f0a\x1e\n")?;
    ///     assert_eq!(record.iter().len(), 1);
    ///
    ///     let result =
    ///         StringRecord::from_bytes(b"003@ \x1f0\x00\x9f\x1e\n");
    ///     assert!(result.is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
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

impl<'a> DerefMut for StringRecord<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
