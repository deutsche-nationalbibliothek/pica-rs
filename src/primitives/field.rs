use std::io::{self, Write};
use std::iter;
use std::str::Utf8Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use winnow::Parser;

use super::parse::parse_field_ref;
use super::{
    Level, Occurrence, OccurrenceRef, ParsePicaError, Subfield,
    SubfieldRef, Tag, TagRef,
};

/// An immutable field.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldRef<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(super) tag: TagRef<'a>,
    pub(super) occurrence: Option<OccurrenceRef<'a>>,
    pub(super) subfields: Vec<SubfieldRef<'a>>,
}

impl<'a> FieldRef<'a> {
    /// Creates a new [FieldRef].
    ///
    /// # Errors
    ///
    /// This function fails if either the tag, occcurrence or any
    /// subfield is nvalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "123456789X")])?;
    /// assert_eq!(field.tag(), "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<T>(
        tag: &'a str,
        occ: Option<&'a str>,
        subfields: T,
    ) -> Result<Self, ParsePicaError>
    where
        T: IntoIterator<Item = (char, &'a str)>,
    {
        let tag = TagRef::new(tag)?;
        let occurrence = if let Some(value) = occ {
            Some(OccurrenceRef::new(value)?)
        } else {
            None
        };

        let subfields = subfields
            .into_iter()
            .map(|(code, value)| SubfieldRef::new(code, value))
            .collect::<Result<Vec<SubfieldRef<'a>>, _>>()?;

        Ok(Self {
            tag,
            occurrence,
            subfields,
        })
    }

    /// Creates an [FieldRef] from a byte slice.
    ///
    /// # Errors
    ///
    /// This function fails if the given byte slice is not a proper
    /// PICA+ field.
    ///
    /// ```rust
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field =
    ///     FieldRef::from_bytes(b"003@ \x1f0123456789X\x1e").unwrap();
    /// assert_eq!(field.tag(), "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_bytes<B>(field: &'a B) -> Result<Self, ParsePicaError>
    where
        B: AsRef<[u8]> + ?Sized,
    {
        let bytes = field.as_ref();

        parse_field_ref.parse(bytes).map_err(|_| {
            ParsePicaError(format!("invalid field '{bytes:?}'"))
        })
    }

    /// Returns a reference to the [TagRef] of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "123456789X")])?;
    /// assert_eq!(field.tag(), "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn tag(&self) -> &TagRef<'a> {
        &self.tag
    }

    /// Returns a reference to the [OccurrenceRef] of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{FieldRef, OccurrenceRef};
    ///
    /// let field =
    ///     FieldRef::new("003@", Some("01"), vec![('0', "123456789X")])?;
    /// assert_eq!(field.occurrence(), Some(&OccurrenceRef::new("01")?));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn occurrence(&self) -> Option<&OccurrenceRef<'a>> {
        self.occurrence.as_ref()
    }

    /// Returns a reference to the [SubfieldRef]s of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "123456789X")])?;
    /// let subfields = field.subfields();
    /// assert_eq!(subfields.len(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn subfields(&self) -> &[SubfieldRef<'a>] {
        &self.subfields
    }

    /// Checks whether a [FieldRef] contains a [SubfieldRef] with the
    /// given code or not.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "123456789X")])?;
    /// assert!(!field.contains('a'));
    /// assert!(field.contains('0'));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn contains(&self, code: char) -> bool {
        self.subfields
            .iter()
            .any(|subfield| *subfield.code() == code)
    }

    /// Searches for the first [SubfieldRef] that satisfies the given
    /// predicate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{FieldRef, SubfieldRef};
    ///
    /// let field =
    ///     FieldRef::new("012A", None, vec![('a', "b"), ('c', "d")])?;
    ///
    /// assert!(field.find(|subfield| subfield.code() == 'b').is_none());
    ///
    /// let subfield =
    ///     field.find(|subfield| subfield.code() == 'a').unwrap();
    /// assert_eq!(subfield, &SubfieldRef::new('a', "b")?);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn find<F>(&self, predicate: F) -> Option<&SubfieldRef>
    where
        F: Fn(&&SubfieldRef) -> bool,
    {
        self.subfields().iter().find(predicate)
    }

    /// Returns the level of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::{FieldRef, Level};
    ///
    /// let field = FieldRef::from_bytes(b"012A/01 \x1fab\x1fcd\x1e")?;
    /// assert_eq!(field.level(), Level::Main);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn level(&self) -> Level {
        self.tag.level()
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the field
    /// contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::from_bytes(b"003@ \x1f0123\x1e")?;
    /// assert!(field.validate().is_ok());
    ///
    /// let field = FieldRef::from_bytes(b"003@ \x1f0\x00\x9F\x1e")?;
    /// assert!(field.validate().is_err());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), Utf8Error> {
        for subfield in self.subfields.iter() {
            subfield.validate()?;
        }

        Ok(())
    }

    /// Write the [FieldRef] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::primitives::FieldRef;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let field = FieldRef::from_bytes(b"012A/01 \x1fab\x1fcd\x1e")?;
    /// field.write_to(&mut writer);
    ///
    /// assert_eq!(
    ///     String::from_utf8(writer.into_inner())?,
    ///     "012A/01 \x1fab\x1fcd\x1e"
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        self.tag.write_to(out)?;

        if let Some(ref occ) = self.occurrence {
            occ.write_to(out)?;
        }

        write!(out, " ")?;

        for subfield in self.subfields.iter() {
            subfield.write_to(out)?;
        }

        write!(out, "\x1e")
    }
}

impl<'a> IntoIterator for &'a FieldRef<'a> {
    type Item = &'a FieldRef<'a>;
    type IntoIter = iter::Once<Self::Item>;

    /// Creates an iterator from a single field. The iterator just
    /// returns the field once.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "abc")])?;
    /// let mut iter = field.into_iter();
    ///
    /// assert_eq!(iter.next(), Some(&field));
    /// assert_eq!(iter.next(), None);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        iter::once(self)
    }
}

/// A mutable field.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Field {
    tag: Tag,
    occurrence: Option<Occurrence>,
    subfields: Vec<Subfield>,
}

impl Field {
    /// Creates a new [Field].
    ///
    /// # Errors
    ///
    /// This function fails if either the tag, occcurrence or any
    /// subfield is nvalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::primitives::Field;
    ///
    /// let _field = Field::new("003@", None, vec![('0', "123456789X")])?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new<'a, T>(
        tag: &str,
        occ: Option<&str>,
        subfields: T,
    ) -> Result<Self, ParsePicaError>
    where
        T: IntoIterator<Item = (char, &'a str)>,
    {
        let tag = Tag::new(tag)?;
        let occurrence = if let Some(value) = occ {
            Some(Occurrence::new(value)?)
        } else {
            None
        };

        let subfields = subfields
            .into_iter()
            .map(|(code, value)| Subfield::new(code, value))
            .collect::<Result<Vec<Subfield>, _>>()?;

        Ok(Self {
            tag,
            occurrence,
            subfields,
        })
    }

    /// Write the [Field] into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::primitives::Field;
    ///
    /// let mut writer = Cursor::new(Vec::<u8>::new());
    /// let field =
    ///     Field::new("012A", Some("01"), vec![('a', "b"), ('c', "d")])?;
    /// field.write_to(&mut writer);
    ///
    /// assert_eq!(
    ///     String::from_utf8(writer.into_inner())?,
    ///     "012A/01 \x1fab\x1fcd\x1e"
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        self.tag.write_to(out)?;

        if let Some(ref occ) = self.occurrence {
            occ.write_to(out)?;
        }

        write!(out, " ")?;

        for subfield in self.subfields.iter() {
            subfield.write_to(out)?;
        }

        write!(out, "\x1e")
    }
}

impl From<FieldRef<'_>> for Field {
    fn from(field: FieldRef<'_>) -> Self {
        let FieldRef {
            tag,
            occurrence,
            subfields,
        } = field;

        Field {
            tag: tag.into(),
            occurrence: occurrence.map(Occurrence::from),
            subfields: subfields
                .into_iter()
                .map(Subfield::from)
                .collect(),
        }
    }
}

impl PartialEq<Field> for FieldRef<'_> {
    fn eq(&self, field: &Field) -> bool {
        let occ_eq = match (&self.occurrence, &field.occurrence) {
            (Some(lhs), Some(rhs)) => lhs == rhs,
            (None, None) => true,
            _ => false,
        };

        self.tag == field.tag
            && occ_eq
            && self.subfields == field.subfields
    }
}

impl PartialEq<FieldRef<'_>> for Field {
    #[inline]
    fn eq(&self, other: &FieldRef<'_>) -> bool {
        other == self
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Field {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let tag = Tag::arbitrary(g);
        let occurrence = Option::<Occurrence>::arbitrary(g);
        let subfields = (0..g.size())
            .map(|_| Subfield::arbitrary(g))
            .collect::<Vec<Subfield>>();

        Self {
            tag,
            occurrence,
            subfields,
        }
    }
}
