use std::io::{self, Write};
use std::iter;
use std::str::Utf8Error;

use winnow::Parser;

use super::parse_field_ref;
use crate::{
    Level, Occurrence, OccurrenceRef, ParsePicaError, Subfield,
    SubfieldRef, Tag, TagRef,
};

/// An immutable PICA+ field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldRef<'a> {
    pub(super) tag: TagRef<'a>,
    pub(super) occurrence: Option<OccurrenceRef<'a>>,
    pub(super) subfields: Vec<SubfieldRef<'a>>,
}

/// A mutable PICA+ field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    tag: Tag,
    occurrence: Option<Occurrence>,
    subfields: Vec<Subfield>,
}

impl<'a> FieldRef<'a> {
    /// Create a new field.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::{FieldRef, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::new("012A", None, vec![('0', "abc")]);
    ///
    ///     assert_eq!(field.tag(), b"012A");
    ///     assert_eq!(field.subfields().len(), 1);
    ///     assert!(field.occurrence().is_none());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<B: ?Sized + AsRef<[u8]>>(
        tag: &'a B,
        occurrence: Option<&'a B>,
        subfields: Vec<(char, &'a B)>,
    ) -> Self {
        let occurrence = occurrence.map(OccurrenceRef::new);
        let subfields: Vec<SubfieldRef<'a>> = subfields
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()
            .expect("valid subfields");

        Self {
            tag: TagRef::new(tag),
            occurrence,
            subfields,
        }
    }

    /// Creates an immutable PICA+ field from a byte slice.
    ///
    /// If an invalid field is given, an error is returned.
    ///
    /// ```rust
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::from_bytes(b"003@ \x1f0123456789X\x1e").unwrap();
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParsePicaError> {
        Self::try_from(bytes)
    }

    /// Returns the tag of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::{FieldRef, TagRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::from_bytes(b"003@ \x1f0123456789X\x1e").unwrap();
    ///     assert_eq!(field.tag(), &TagRef::new("003@"));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn tag(&self) -> &TagRef {
        &self.tag
    }

    /// Returns a reference to the occurrence of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::{FieldRef, Occurrence};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::new("012A", Some("01"), vec![]);
    ///     let occurrence = field.occurrence().unwrap();
    ///     assert_eq!(*occurrence, "01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn occurrence(&self) -> Option<&OccurrenceRef> {
        self.occurrence.as_ref()
    }

    /// Returns the subfields of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::new(
    ///         "012A",
    ///         Some("01"),
    ///         vec![('a', "b"), ('c', "d")],
    ///     );
    ///
    ///     assert_eq!(field.subfields().len(), 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn subfields(&self) -> &Vec<SubfieldRef> {
        self.subfields.as_ref()
    }

    /// Returns `true` if the field contains a subfield with the given
    /// code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::from_bytes(b"003@ \x1f0123456789X\x1e").unwrap();
    ///     assert!(field.contains('0'));
    ///     assert!(!field.contains('a'));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn contains(&self, code: char) -> bool {
        self.subfields
            .iter()
            .any(|subfield| *subfield.code() == code)
    }

    /// Searches for the first subfield that satisfies the given
    /// predicate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::{FieldRef, SubfieldRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::new(
    ///         "012A",
    ///         Some("01"),
    ///         vec![('a', "b"), ('c', "d")],
    ///     );
    ///
    ///     assert_eq!(
    ///         field.find(|subfield| subfield.code() == 'a').unwrap(),
    ///         &SubfieldRef::new('a', "b")?
    ///     );
    ///
    ///     assert!(field
    ///         .find(|subfield| subfield.code() == 'b')
    ///         .is_none());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn find<F>(&self, predicate: F) -> Option<&SubfieldRef>
    where
        F: Fn(&&SubfieldRef) -> bool,
    {
        self.subfields().iter().find(predicate)
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the field
    /// contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::from_bytes(b"003@ \x1f0123\x1e")?;
    ///     assert!(field.validate().is_ok());
    ///
    ///     let field = FieldRef::from_bytes(b"003@ \x1f0\x00\x9F\x1e")?;
    ///     ///     assert!(field.validate().is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), Utf8Error> {
        for subfield in self.subfields() {
            subfield.validate()?;
        }

        Ok(())
    }

    /// Write the field into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let field = FieldRef::from_bytes(b"012A/01 \x1fab\x1fcd\x1e")?;
    ///     field.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #    String::from_utf8(writer.into_inner())?,
    ///     #    "012A/01 \x1fab\x1fcd\x1e"
    ///     # );
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "{}", self.tag)?;
        self.occurrence().map(|o| o.write_to(out));

        write!(out, " ")?;

        for subfield in self.subfields.iter() {
            subfield.write_to(out)?;
        }

        write!(out, "\x1e")
    }

    /// Returns the level of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record_v1::{FieldRef, Level};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::from_bytes(b"012A/01 \x1fab\x1fcd\x1e")?;
    ///     assert_eq!(field.level(), Level::Main);
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn level(&self) -> Level {
        self.tag.level()
    }
}

impl<'a> TryFrom<&'a [u8]> for FieldRef<'a> {
    type Error = ParsePicaError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        parse_field_ref
            .parse(value)
            .map_err(|_| ParsePicaError::InvalidField)
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
    /// use pica_record_v1::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::new("003@", None, vec![('0', "abc")]);
    ///     let mut iter = field.into_iter();
    ///
    ///     assert_eq!(iter.next(), Some(&field));
    ///     assert_eq!(iter.next(), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        iter::once(self)
    }
}

impl From<FieldRef<'_>> for Field {
    fn from(other: FieldRef<'_>) -> Self {
        let FieldRef {
            tag,
            occurrence,
            subfields,
        } = other;

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

impl Field {
    /// Write the field into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record_v1::{Field, FieldRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     let field: Field =
    ///         FieldRef::from_bytes(b"012A/01 \x1fab\x1fcd\x1e")?.into();
    ///     field.write_to(&mut writer);
    ///     #
    ///     # assert_eq!(
    ///     #    String::from_utf8(writer.into_inner())?,
    ///     #    "012A/01 \x1fab\x1fcd\x1e"
    ///     # );
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        let _ = out.write(self.tag.as_bytes())?;

        if let Some(ref o) = self.occurrence {
            o.write_to(out)?;
        }

        write!(out, " ")?;

        for subfield in self.subfields.iter() {
            subfield.write_to(out)?;
        }

        write!(out, "\x1e")
    }
}

#[cfg(feature = "arbitrary")]
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
