use std::io::{self, Write};

use bstr::{BStr, BString};
use nom::character::complete::char;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::tuple;
use nom::Finish;

use crate::occurrence::parse_occurrence_ref;
use crate::parser::{ParseResult, RS, SP};
use crate::subfield::parse_subfield_ref;
use crate::tag::parse_tag_ref;
use crate::{
    Occurrence, OccurrenceRef, ParsePicaError, Subfield, SubfieldRef,
    Tag, TagRef,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldRef<'a> {
    pub(crate) tag: TagRef<'a>,
    pub(crate) occurrence: Option<OccurrenceRef<'a>>,
    pub(crate) subfields: Vec<SubfieldRef<'a>>,
}

impl<'a> FieldRef<'a> {
    /// Create a new field reference.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::new("003@", None, vec![('0', "123456789X")]);
    ///     assert_eq!(field.tag(), "003@");
    ///     assert!(field.occurrence().is_none());
    ///     assert_eq!(field.subfields().len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: Into<&'a BStr>>(
        tag: T,
        occurrence: Option<T>,
        subfields: Vec<(char, T)>,
    ) -> FieldRef<'a> {
        let occurrence =
            occurrence.map(|digits| OccurrenceRef::new(digits.into()));
        let subfields = subfields
            .into_iter()
            .map(|(code, value)| SubfieldRef::new(code, value))
            .collect();

        FieldRef {
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
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     assert!(
    ///         FieldRef::from_bytes(b"003@ \x1f0123456789X\x1e").is_ok()
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParsePicaError> {
        parse_field_ref(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidField)
            .map(|(_, field)| field)
    }

    /// Returns the tag of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::new("003@", None, vec![('0', "123456789X")]);
    ///     assert_eq!(field.tag(), "003@");
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
    /// use pica_record::{FieldRef, OccurrenceRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::new("012A", Some("01"), vec![]);
    ///     let occurrence = field.occurrence().unwrap();
    ///     assert_eq!(occurrence, "01");
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
    /// use pica_record::FieldRef;
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

    /// Converts the immutable subfield into its mutable counterpart by
    /// consuming the source.
    pub fn into_owned(self) -> Field {
        self.into()
    }

    /// Converts the immutable subfield into its mutable counterpart.
    pub fn to_owned(&self) -> Field {
        self.clone().into()
    }

    /// Write the field into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use pica_record::FieldRef;
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
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "{}", self.tag.0)?;
        self.occurrence().map(|o| o.write_to(out));
        write!(out, " ")?;
        for subfield in self.subfields.iter() {
            subfield.write_to(out)?;
        }
        write!(out, "\x1e")
    }
}

/// Parse a PICA+ field (read-only).
pub fn parse_field_ref(i: &[u8]) -> ParseResult<FieldRef> {
    map(
        tuple((
            parse_tag_ref,
            opt(parse_occurrence_ref),
            char(SP as char),
            many0(parse_subfield_ref),
            char(RS as char),
        )),
        |(tag, occurrence, _, subfields, _)| FieldRef {
            tag,
            occurrence,
            subfields,
        },
    )(i)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    tag: Tag,
    occurrence: Option<Occurrence>,
    subfields: Vec<Subfield>,
}

impl Field {
    /// Create a new field.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = Field::new("003@", None, vec![('0', "123456789X")]);
    ///     assert_eq!(field.tag(), "003@");
    ///     assert!(field.occurrence().is_none());
    ///     assert_eq!(field.subfields().len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: Into<BString>>(
        tag: T,
        occurrence: Option<T>,
        subfields: Vec<(char, T)>,
    ) -> Self {
        let occurrence =
            occurrence.map(|digits| Occurrence::new(digits.into()));
        let subfields = subfields
            .into_iter()
            .map(|(code, value)| Subfield::new(code, value))
            .collect();

        Field {
            tag: Tag::new(tag),
            occurrence,
            subfields,
        }
    }

    /// Returns the tag of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = Field::new("003@", None, vec![('0', "123456789X")]);
    ///     assert_eq!(field.tag(), "003@");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    /// Returns a reference to the occurrence of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{Field, Occurrence};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = Field::new("012A", Some("01"), vec![]);
    ///     let occurrence = field.occurrence().unwrap();
    ///     assert_eq!(occurrence, "01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn occurrence(&self) -> Option<&Occurrence> {
        self.occurrence.as_ref()
    }

    /// Returns the subfields of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = Field::new(
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
    pub fn subfields(&self) -> &Vec<Subfield> {
        self.subfields.as_ref()
    }
}

impl From<FieldRef<'_>> for Field {
    fn from(field_ref: FieldRef<'_>) -> Self {
        let FieldRef {
            tag,
            occurrence,
            subfields,
        } = field_ref;

        Field {
            tag: tag.into(),
            occurrence: occurrence.map(|o| o.into()),
            subfields: subfields
                .into_iter()
                .map(|s| s.into())
                .collect(),
        }
    }
}
