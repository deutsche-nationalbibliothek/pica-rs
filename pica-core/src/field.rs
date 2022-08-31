//! This module contains data structures and functions related to
//! PICA+ fields.

use std::fmt;
use std::io::Write;
use std::str::{FromStr, Utf8Error};

use bstr::BString;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use nom::character::complete::char;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::{preceded, terminated, tuple};
use nom::Finish;

use crate::parser::{parse_occurrence, parse_subfield, parse_tag};
use crate::{
    Occurrence, OccurrenceRef, ParseError, ParseResult, Subfield, SubfieldRef,
    Tag, TagRef,
};

/// An immutable PICA+ field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldRef<'a> {
    tag: TagRef<'a>,
    occurrence: Option<OccurrenceRef<'a>>,
    subfields: Vec<SubfieldRef<'a>>,
}

/// Parse a PICA+ field.
#[inline]
pub fn parse_field<'a>(i: &'a [u8]) -> ParseResult<FieldRef<'a>> {
    const RS: char = '\x1E';
    const SP: char = '\x20';

    let (i, (tag, occurrence, subfields)) = terminated(
        tuple((
            parse_tag,
            opt(parse_occurrence),
            preceded(char(SP), many0(parse_subfield)),
        )),
        char(RS),
    )(i)?;

    Ok((
        i,
        FieldRef {
            tag,
            occurrence,
            subfields,
        },
    ))
}

impl<'a> FieldRef<'a> {
    /// Creates an immutable PICA+ field from a byte slice.
    ///
    /// ```rust
    /// use pica_core::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(FieldRef::from_bytes(b"003@ \x1f0123456789X\x1e").is_ok());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ParseError> {
        Ok(match parse_field(data).finish() {
            Ok((_, field)) => field,
            _ => return Err(ParseError::InvalidField),
        })
    }
}

/// An mutable PICA+ field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    tag: Tag,
    occurrence: Option<Occurrence>,
    subfields: Vec<Subfield>,
}

impl Field {
    /// Creates a new `Field`
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"003@")?,
    ///         None,
    ///         vec![Subfield::from_bytes(b"\x1f0123456789X")?],
    ///     );
    ///
    ///     assert_eq!(field.tag(), &Tag::from_bytes(b"003@")?);
    ///     assert_eq!(field.len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(
        tag: Tag,
        occurrence: Option<Occurrence>,
        subfields: Vec<Subfield>,
    ) -> Self {
        Self {
            tag,
            occurrence,
            subfields,
        }
    }
    /// Creates an PICA+ field from a byte slice.
    ///
    /// ```rust
    /// use pica_core::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(Field::from_bytes(b"002@ \x1f0Oaf\x1e").is_ok());
    ///     assert!(Field::from_bytes(b"002@\x1fOaf\x1e").is_err());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        Ok(FieldRef::from_bytes(data)?.into())
    }

    /// Get a reference to the field's tag.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(Tag::from_bytes(b"003@")?, None, vec![]);
    ///     assert_eq!(field.tag(), &Tag::from_bytes(b"003@")?);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    /// Get a reference to the field's occurrence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Occurrence, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"012A")?,
    ///         Some(Occurrence::from_bytes(b"/00")?),
    ///         vec![],
    ///     );
    ///     assert_eq!(field.occurrence(), Some(&Occurrence::from_bytes(b"/00")?));
    ///     Ok(())
    /// }
    /// ```
    pub fn occurrence(&self) -> Option<&Occurrence> {
        self.occurrence.as_ref()
    }

    /// Get a reference to the field's occurrence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"012A")?,
    ///         None,
    ///         vec![Subfield::from_bytes(b"\x1f0123456789X")?],
    ///     );
    ///     assert_eq!(
    ///         field.subfields(),
    ///         &[Subfield::from_bytes(b"\x1f0123456789X")?]
    ///     );
    ///     Ok(())
    /// }
    /// ```
    pub fn subfields(&self) -> &Vec<Subfield> {
        &self.subfields
    }

    /// Returns the number of subfields.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"012A")?,
    ///         None,
    ///         vec![Subfield::from_bytes(b"\x1f0123456789X")?],
    ///     );
    ///     assert_eq!(field.len(), 1);
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.subfields().len()
    }

    /// Returns `true` if the field is empty (no subfields), otherwise `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(Tag::from_bytes(b"012A")?, None, vec![]);
    ///     assert!(field.is_empty());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.subfields().is_empty()
    }

    /// Returns an iterator over subfields
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::from_bytes(b"\x1f0123456789X")?;
    ///     let field =
    ///         Field::new(Tag::from_bytes(b"012A")?, None, vec![subfield.clone()]);
    ///
    ///     let mut iter = field.iter();
    ///     assert_eq!(iter.next(), Some(&subfield));
    ///     assert_eq!(iter.next(), None);
    ///     Ok(())
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Subfield> {
        self.subfields().iter()
    }

    /// Returns `true` if the `Field` contains a `Subfield` with the specified
    ///  code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"003@")?,
    ///         None,
    ///         vec![Subfield::from_bytes(b"\x1f0123456789X")?],
    ///     );
    ///
    ///     assert_eq!(field.contains_code('0'), true);
    ///     assert_eq!(field.contains_code('1'), false);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn contains_code(&self, code: char) -> bool {
        self.iter().any(|x| x.code() == code)
    }

    /// Returns a list of references to all `Subfields` of the given subfield
    /// code.
    ///
    /// If no subfield exists `None` is returned.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"003@")?,
    ///         None,
    ///         vec![Subfield::from_bytes(b"\x1f0123456789X")?],
    ///     );
    ///
    ///     assert_eq!(
    ///         field.get('0'),
    ///         Some(vec![&Subfield::from_bytes(b"\x1f0123456789X")?])
    ///     );
    ///     assert_eq!(field.get('1'), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn get(&self, code: char) -> Option<Vec<&Subfield>> {
        let subfields = self
            .iter()
            .filter(|x| x.code() == code)
            .collect::<Vec<&Subfield>>();

        if !subfields.is_empty() {
            Some(subfields)
        } else {
            None
        }
    }

    /// Returns the first subfield value
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"003@")?,
    ///         None,
    ///         vec![
    ///             Subfield::from_bytes(b"\x1faabc")?,
    ///             Subfield::from_bytes(b"\x1fadef")?,
    ///             Subfield::from_bytes(b"\x1fahij")?,
    ///         ],
    ///     );
    ///
    ///     assert_eq!(field.first('a').unwrap(), "abc");
    ///     assert_eq!(field.first('1'), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn first(&self, code: char) -> Option<&BString> {
        self.iter()
            .filter(|x| x.code() == code)
            .map(|x| x.value())
            .next()
    }

    /// Returns the all subfield values for the subfield code
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"003@")?,
    ///         None,
    ///         vec![
    ///             Subfield::from_bytes(b"\x1faabc")?,
    ///             Subfield::from_bytes(b"\x1fadef")?,
    ///             Subfield::from_bytes(b"\x1fahij")?,
    ///         ],
    ///     );
    ///
    ///     assert_eq!(field.all('a').unwrap(), vec!["abc", "def", "hij"]);
    ///     assert_eq!(field.all('b'), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn all(&self, code: char) -> Option<Vec<&BString>> {
        let result = self
            .iter()
            .filter(|x| x.code() == code)
            .map(|x| x.value())
            .collect::<Vec<&BString>>();

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }

    /// Returns `true` if and only if all subfield values are valid UTF-8.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"003@")?,
    ///         None,
    ///         vec![Subfield::from_bytes(b"\x1f0123456789X")?],
    ///     );
    ///
    ///     assert_eq!(field.validate().is_ok(), true);
    ///
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"003@")?,
    ///         None,
    ///         vec![
    ///             Subfield::from_bytes(b"\x1f0234567890X")?,
    ///             Subfield::from_bytes(&[b'\x1f', b'0', 0, 159])?,
    ///             Subfield::from_bytes(b"\x1f2123456789X")?,
    ///         ],
    ///     );
    ///
    ///     assert_eq!(field.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<(), Utf8Error> {
        for subfield in &self.subfields {
            subfield.validate()?;
        }

        Ok(())
    }

    /// Write the field into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Tag, Occurrence, Subfield, Field};
    /// use std::io::Cursor;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"012A")?,
    ///         Some(Occurrence::from_bytes(b"/001")?),
    ///         vec![Subfield::from_bytes(b"\x1f0123456789X")?],
    ///     );
    ///
    ///     let mut writer = Cursor::new(Vec::<u8>::new());
    ///     field.write(&mut writer)?;
    ///
    ///     # let result = String::from_utf8(writer.into_inner())?;
    ///     # assert_eq!(result, String::from("012A/001 \x1f0123456789X\x1e"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write(&self, writer: &mut dyn Write) -> Result<(), std::io::Error> {
        writer.write_all(self.tag.as_slice())?;

        if let Some(ref occurrence) = self.occurrence {
            write!(writer, "{}", occurrence)?;
        }

        writer.write_all(&[b' '])?;

        for subfield in &self.subfields {
            subfield.write(writer)?;
        }

        writer.write_all(&[b'\x1e'])?;
        Ok(())
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
            occurrence: occurrence.map(Into::into),
            subfields: subfields.into_iter().map(Into::into).collect(),
        }
    }
}

impl FromStr for Field {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes())
    }
}

impl fmt::Display for Field {
    /// Format the field in a human-readable format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_core::{Field, Occurrence, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::from_bytes(b"012A")?,
    ///         Some(Occurrence::from_bytes(b"/01")?),
    ///         vec![
    ///             Subfield::from_bytes(b"\x1f0123456789X")?,
    ///             Subfield::from_bytes(b"\x1fafoo")?,
    ///         ],
    ///     );
    ///
    ///     assert_eq!(format!("{}", field), "012A/01 $0123456789X$afoo");
    ///
    ///     Ok(())
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)?;

        if let Some(ref occurrence) = self.occurrence {
            write!(f, "{}", occurrence)?;
        }

        if !self.is_empty() {
            let subfields = self
                .iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<_>>()
                .join("");

            write!(f, " {}", subfields)?;
        }

        Ok(())
    }
}
impl Serialize for Field {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Field", 3)?;
        state.serialize_field("tag", &self.tag.to_string())?;
        if let Some(occurrence) = self.occurrence() {
            let occurrence = occurrence.to_string();
            state.serialize_field("occurrence", &occurrence[1..])?;
        }

        state.serialize_field("subfields", &self.subfields)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TestResult;

    use nom_test_helpers::prelude::*;

    #[test]
    fn test_parse_tag() -> TestResult {
        assert_done_and_eq!(
            parse_field(b"003@ \x1f0123456789X\x1fabc\x1e"),
            FieldRef {
                tag: TagRef::from_bytes(b"003@")?,
                occurrence: None,
                subfields: vec![
                    SubfieldRef::from_bytes(b"\x1f0123456789X")?,
                    SubfieldRef::from_bytes(b"\x1fabc")?
                ]
            }
        );

        Ok(())
    }
}
