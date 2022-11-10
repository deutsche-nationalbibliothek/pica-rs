use std::fmt::Display;
use std::io::{self, Write};
use std::str::Utf8Error;

use bstr::{BStr, BString};
use nom::character::complete::char;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::tuple;
use nom::Finish;

use crate::occurrence::parse_occurrence;
use crate::parser::{ParseResult, RS, SP};
use crate::subfield::parse_subfield;
use crate::tag::parse_tag;
use crate::{Occurrence, ParsePicaError, Subfield, Tag};

/// A PICA+ field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field<T: AsRef<[u8]>> {
    tag: Tag<T>,
    occurrence: Option<Occurrence<T>>,
    subfields: Vec<Subfield<T>>,
}

/// A immutable PICA+ field.
pub type FieldRef<'a> = Field<&'a BStr>;

/// A mutable PICA+ field.
pub type FieldMut = Field<BString>;

impl<'a, T: AsRef<[u8]>> Field<T> {
    /// Returns the tag of the field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{FieldRef, TagRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::new("003@", None, vec![('0', "123456789X")]);
    ///     assert_eq!(field.tag(), &TagRef::new("003@"));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn tag(&self) -> &Tag<T> {
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
    ///     assert_eq!(*occurrence, "01");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn occurrence(&self) -> Option<&Occurrence<T>> {
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
    pub fn subfields(&self) -> &Vec<Subfield<T>> {
        self.subfields.as_ref()
    }
}

impl<'a, T: AsRef<[u8]> + From<&'a BStr> + Display> Field<T> {
    /// Create a new field.
    ///
    /// # Panics
    ///
    /// This method panics if a parameter is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::{FieldRef, TagRef};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::new("003@", None, vec![('0', "123456789X")]);
    ///     assert_eq!(field.tag(), &TagRef::new("003@"));
    ///     assert!(field.occurrence().is_none());
    ///     assert_eq!(field.subfields().len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<U: Into<T>>(
        tag: U,
        occurrence: Option<U>,
        subfields: Vec<(char, U)>,
    ) -> Self {
        let occurrence =
            occurrence.map(|digits| Occurrence::new(digits));
        let subfields = subfields
            .into_iter()
            .map(|(code, value)| Subfield::new(code, value))
            .collect();

        Self {
            tag: Tag::new(tag.into()),
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
        parse_field(data)
            .finish()
            .map_err(|_| ParsePicaError::InvalidField)
            .map(|(_, (tag, occurrence, subfields))| Self {
                tag: Tag::from_unchecked(tag),
                occurrence: occurrence.map(Occurrence::from_unchecked),
                subfields: subfields
                    .into_iter()
                    .map(|(code, value)| Subfield {
                        code,
                        value: value.into(),
                    })
                    .collect(),
            })
    }

    /// Returns an [`std::str::Utf8Error`](Utf8Error) if the field
    /// contains invalid UTF-8 data, otherwise the unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field = FieldRef::from_bytes(b"003@ \x1f0123\x1e")?;
    ///     assert!(field.validate().is_ok());
    ///
    ///     let field = FieldRef::from_bytes(b"003@ \x1f0\x00\x9F\x1e")?;
    ///     assert!(field.validate().is_err());
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
        write!(out, "{}", *self.tag)?;
        self.occurrence().map(|o| o.write_to(out));
        write!(out, " ")?;
        for subfield in self.subfields.iter() {
            subfield.write_to(out)?;
        }
        write!(out, "\x1e")
    }
}

impl<'a> From<FieldRef<'a>> for FieldMut {
    #[inline]
    fn from(field: FieldRef<'a>) -> Self {
        let FieldRef {
            tag,
            occurrence,
            subfields,
        } = field;

        FieldMut {
            tag: tag.into(),
            occurrence: occurrence.map(|o| o.into()),
            subfields: subfields
                .into_iter()
                .map(|s| s.into())
                .collect(),
        }
    }
}

impl<'a> FieldRef<'a> {
    /// Converts the immutable tag into its mutable counterpart by
    /// consuming the source.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let tag = TagRef::new("003@").into_owned();
    ///     assert_eq!(tag, "003@");
    ///     Ok(())
    /// }
    /// ```
    pub fn into_owned(self) -> FieldMut {
        self.into()
    }

    /// Converts the immutable tag into its mutable counterpart.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let tag = TagRef::new("003@").to_owned();
    ///     assert_eq!(tag, "003@");
    ///     Ok(())
    /// }
    /// ```
    pub fn to_owned(&self) -> FieldMut {
        self.clone().into()
    }
}

pub(crate) type RawField<'a> =
    (&'a BStr, Option<&'a BStr>, Vec<(char, &'a BStr)>);

/// Parse a PICA+ field.
pub fn parse_field(i: &[u8]) -> ParseResult<RawField> {
    map(
        tuple((
            parse_tag,
            opt(parse_occurrence),
            char(SP as char),
            many0(parse_subfield),
            char(RS as char),
        )),
        |(tag, occurrence, _, subfields, _)| {
            (tag, occurrence, subfields)
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_parse_field_value() {
        assert_done!(parse_field(b"012A/01 \x1fabc\x1e"),);
        assert_done!(parse_field(b"012A \x1fabc\x1e"),);
        assert_done!(parse_field(b"012A \x1e"),);

        assert_error!(parse_field(b"012!/01 \x1fabc\x1e"));
        assert_error!(parse_field(b"012A/0! \x1fabc\x1e"));
        assert_error!(parse_field(b"012A/00\x1fabc\x1e"));
        assert_error!(parse_field(b"012A/00 abc\x1e"));
        assert_error!(parse_field(b"012A/00 \x1f!bc\x1e"));
        assert_error!(parse_field(b"012A/00 \x1fabc"));
    }
}
