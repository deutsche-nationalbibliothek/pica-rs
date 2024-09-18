use std::fmt::{self, Display};
use std::io::{self, Write};

use bstr::{BStr, BString};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use winnow::Parser;

use super::parse::parse_occurrence_ref;
use super::ParsePicaError;

/// An immutable occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OccurrenceRef<'a>(
    #[cfg_attr(feature = "serde", serde(borrow))] &'a BStr,
);

impl<'a> OccurrenceRef<'a> {
    /// Create a new [OccurrenceRef] from a string slice.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::new("001")?;
    /// assert_eq!(occurrence, "001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(occ: &'a str) -> Result<Self, ParsePicaError> {
        Self::from_bytes(occ.as_bytes())
    }

    /// Create a new [OccurrenceRef] without checking for validity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the occurrence is valid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::from_unchecked("001");
    /// assert_eq!(occurrence, "001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_unchecked<T: AsRef<[u8]> + ?Sized>(
        occurrence: &'a T,
    ) -> Self {
        Self(occurrence.as_ref().into())
    }

    /// Create a new [OccurrenceRef] from a byte slice.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::from_bytes(b"00")?;
    /// assert_eq!(occurrence, "00");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn from_bytes<B: AsRef<[u8]> + ?Sized>(
        occurrence: &'a B,
    ) -> Result<Self, ParsePicaError> {
        let bytes = occurrence.as_ref();

        parse_occurrence_ref.parse(bytes).map_err(|_| {
            ParsePicaError(format!("invalid occurrence '{bytes:?}'"))
        })
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0
    }

    /// Write the [OccurrenceRef] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::primitives::OccurrenceRef;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let occurrence = OccurrenceRef::new("01")?;
    /// occurrence.write_to(&mut writer);
    ///
    /// assert_eq!(String::from_utf8(writer.into_inner())?, "/01");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "/{}", self.0)
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for OccurrenceRef<'_> {
    /// Compare a [OccurrenceRef] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::from_bytes(b"01")?;
    /// assert_eq!(occurrence, b"01");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn eq(&self, occurrence: &T) -> bool {
        self.0 == occurrence.as_ref()
    }
}

/// A mutable occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Occurrence(BString);

impl Occurrence {
    /// Create a new [OccurrenceRef] from a string slice.
    ///
    /// # Panics
    ///
    /// This method panics if the occurrence is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::OccurrenceRef;
    ///
    /// let occurrence = OccurrenceRef::new("001")?;
    /// assert_eq!(occurrence, "001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(occurrence: &str) -> Result<Self, ParsePicaError> {
        Ok(Self::from(OccurrenceRef::from_bytes(
            occurrence.as_bytes(),
        )?))
    }

    /// Returns the [Occurrence] as a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::Occurrence;
    ///
    /// let occurrence = Occurrence::new("001")?;
    /// assert_eq!(occurrence.as_bytes(), b"001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Returns the length (number of digits) of the [Occurrence].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::Occurrence;
    ///
    /// let occurrence = Occurrence::new("001")?;
    /// assert_eq!(occurrence.len(), 3);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Write the [Occurrence] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::primitives::Occurrence;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let occurrence = Occurrence::new("01")?;
    /// occurrence.write_to(&mut writer);
    ///
    /// assert_eq!(String::from_utf8(writer.into_inner())?, "/01");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "/{}", self.0)
    }
}

impl Display for Occurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<OccurrenceRef<'_>> for Occurrence {
    /// Creates a [Occurrence] from a [OccurrenceRef].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{Occurrence, OccurrenceRef};
    ///
    /// let occ_ref = OccurrenceRef::new("001")?;
    /// let occ = Occurrence::from(occ_ref);
    /// assert_eq!(occ, "001");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn from(occurrence: OccurrenceRef<'_>) -> Self {
        let OccurrenceRef(occ) = occurrence;
        Self(occ.into())
    }
}

impl PartialEq<&str> for Occurrence {
    /// Compares a [Occurrence] with a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::Occurrence;
    ///
    /// let occ = Occurrence::new("999")?;
    /// assert_eq!(occ, "999");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

impl PartialEq<Occurrence> for OccurrenceRef<'_> {
    /// Compares a [OccurrenceRef] with [Occurrence].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{Occurrence, OccurrenceRef};
    ///
    /// let occ_ref = OccurrenceRef::new("999")?;
    /// let occ = Occurrence::new("999")?;
    /// assert_eq!(occ_ref, occ);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn eq(&self, other: &Occurrence) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Occurrence {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let size = *g.choose(&[2, 3]).unwrap();
        let value = (0..size)
            .map(|_| *g.choose(b"0123456789").unwrap())
            .collect::<Vec<u8>>();

        Occurrence(value.into())
    }
}
