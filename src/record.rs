use std::hash::{Hash, Hasher};
use std::io::{self, Cursor, Write};
use std::ops::{Deref, DerefMut};
use std::str::Utf8Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::primitives::{FieldRef, ParsePicaError, RecordRef};

/// A record, that may contain invalid UTF-8 data.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ByteRecord<'a> {
    raw_data: Option<&'a [u8]>,
    record: RecordRef<'a>,
}

impl<'a> ByteRecord<'a> {
    /// Creates a new [ByteRecord] from a byte slice.
    ///
    /// # Errors
    ///
    /// If an invalid record is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::ByteRecord;
    ///
    /// let record = ByteRecord::from_bytes(b"003@ \x1f0abc\x1e\n")?;
    /// assert_eq!(record.fields().len(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes<B: AsRef<[u8]>>(
        bytes: &'a B,
    ) -> Result<Self, ParsePicaError> {
        Ok(Self {
            record: RecordRef::from_bytes(bytes)?,
            raw_data: Some(bytes.as_ref()),
        })
    }

    /// Write the [ByteRecord] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::ByteRecord;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let record = ByteRecord::from_bytes(b"003@ \x1f0a\x1e\n")?;
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
    /// use pica_record::ByteRecord;
    ///
    /// let mut record =
    ///     ByteRecord::from_bytes(b"003@ \x1f0a\x1e002@ \x1f0Olfo\x1e\n")?;
    /// record.retain(|field| field.tag() == "003@");
    /// assert_eq!(record.fields().len(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn retain<F: FnMut(&FieldRef) -> bool>(&mut self, f: F) {
        self.record.retain(f);
        self.raw_data = None;
    }

    /// Returns the SHA-256 hash of the record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::fmt::Write;
    ///
    /// use pica_record::ByteRecord;
    ///
    /// let mut record = ByteRecord::from_bytes(b"012A \x1fa123\x1e\n")?;
    ///
    /// let hash =
    ///     record.sha256().iter().fold(String::new(), |mut out, b| {
    ///         let _ = write!(out, "{b:02x}");
    ///         out
    ///     });
    ///
    /// assert!(hash.starts_with("95e266"));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

impl DerefMut for ByteRecord<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.record
    }
}

impl PartialEq<ByteRecord<'_>> for ByteRecord<'_> {
    /// Compare two [ByteRecord]s.
    ///
    /// # Note
    ///
    /// It's important not to derive [PartialEq] for a [ByteRecord],
    /// because a record might have cached the raw data. There are two
    /// cases to consider: If both records have raw data it is
    /// sufficient to compare these byte slices. Otherwise, bot records
    /// must be compared field by field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::ByteRecord;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let record1 = ByteRecord::from_bytes(b"012A \x1fa123\x1e\n")?;
    /// let record2 = ByteRecord::from(RecordRef::new(vec![(
    ///     "012A",
    ///     None,
    ///     vec![('a', "123")],
    /// )])?);
    ///
    /// assert_eq!(record1, record2);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &ByteRecord<'_>) -> bool {
        match (self.raw_data, other.raw_data) {
            (Some(lhs), Some(rhs)) => lhs == rhs,
            _ => self.record == other.record,
        }
    }
}

impl<'a> From<RecordRef<'a>> for ByteRecord<'a> {
    /// Creates a [ByteRecord] from a [RecordRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::ByteRecord;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let record1 = ByteRecord::from_bytes(b"012A \x1fa123\x1e\n")?;
    /// let record2 = ByteRecord::from(record1);
    /// assert_eq!(record2.fields().len(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn from(record: RecordRef<'a>) -> Self {
        ByteRecord {
            raw_data: None,
            record,
        }
    }
}

impl Hash for ByteRecord<'_> {
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

/// A record, that guarantees valid UTF-8 data.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StringRecord<'a>(
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate)  ByteRecord<'a>,
);

impl<'a> TryFrom<ByteRecord<'a>> for StringRecord<'a> {
    type Error = Utf8Error;

    /// Creates a [StringRecord] from a [ByteRecord].
    ///
    /// # Errors
    ///
    /// If the underlying [ByteRecord] contains invalid UTF-8 sequences,
    /// an [Utf8Error] is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{ByteRecord, StringRecord};
    ///
    /// let record = ByteRecord::from_bytes(b"012A \x1fa123\x1e\n")?;
    /// let record = StringRecord::try_from(record)?;
    /// assert_eq!(record.fields().len(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn try_from(record: ByteRecord<'a>) -> Result<Self, Self::Error> {
        record.validate()?;
        Ok(Self(record))
    }
}

impl<'a> Deref for StringRecord<'a> {
    type Target = ByteRecord<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StringRecord<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
