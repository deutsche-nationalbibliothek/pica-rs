//! This module contains data structures and functions related to
//! PICA+ field.

use std::fmt;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

use bstr::{BString, ByteSlice};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, map, success, value};
use nom::multi::many0;
use nom::sequence::{preceded, terminated, tuple};
use nom::Finish;

use crate::common::{ws, MatcherFlags, ParseResult};
use crate::error::{Error, Result};
use crate::occurrence::{
    parse_occurrence, parse_occurrence_matcher, Occurrence,
};
use crate::subfield::{
    parse_subfield, parse_subfields_matcher, parse_subfields_matcher_simple,
    Subfield,
};
use crate::tag::{parse_tag, parse_tag_matcher, Tag};
use crate::{OccurrenceMatcher, SubfieldsMatcher, TagMatcher};

const RS: char = '\x1E';
const SP: char = '\x20';

/// A PICA+ field, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq)]
pub struct Field {
    pub(crate) tag: Tag,
    pub(crate) occurrence: Option<Occurrence>,
    pub(crate) subfields: Vec<Subfield>,
}

impl Deref for Field {
    type Target = Vec<Subfield>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.subfields
    }
}

impl Field {
    /// Creates a new `Field`
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("003@")?,
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     );
    ///
    ///     assert_eq!(field.tag(), &Tag::new("003@")?);
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

    /// Get a reference to the field's tag.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(Tag::new("003@")?, None, vec![]);
    ///     assert_eq!(field.tag(), &Tag::new("003@")?);
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
    /// use pica::{Field, Occurrence, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field =
    ///         Field::new(Tag::new("012A")?, Some(Occurrence::new("00")?), vec![]);
    ///     assert_eq!(field.occurrence(), Some(&Occurrence::new("00")?));
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
    /// use pica::{Field, Occurrence, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("012A")?,
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     );
    ///     assert_eq!(field.subfields(), &[Subfield::new('0', "123456789X")?]);
    ///     Ok(())
    /// }
    /// ```
    pub fn subfields(&self) -> &[Subfield] {
        &self.subfields
    }

    /// Returns `true` if the `Field` contains a `Subfield` with the specified
    ///  code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("003@")?,
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     );
    ///
    ///     assert_eq!(field.contains_code('0'), true);
    ///     assert_eq!(field.contains_code('1'), false);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn contains_code(&self, code: char) -> bool {
        self.iter().any(|x| x.code == code)
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
    /// use pica::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("003@")?,
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     );
    ///
    ///     assert_eq!(
    ///         field.get('0'),
    ///         Some(vec![&Subfield::new('0', "123456789X")?])
    ///     );
    ///     assert_eq!(field.get('1'), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn get(&self, code: char) -> Option<Vec<&Subfield>> {
        let subfields = self
            .iter()
            .filter(|x| x.code == code)
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
    /// use pica::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("003@")?,
    ///         None,
    ///         vec![
    ///             Subfield::new('a', "abc")?,
    ///             Subfield::new('a', "def")?,
    ///             Subfield::new('a', "hij")?,
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
            .filter(|x| x.code == code)
            .map(|x| &x.value)
            .next()
    }

    /// Returns the all subfield values for the subfield code
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("003@")?,
    ///         None,
    ///         vec![
    ///             Subfield::new('a', "abc")?,
    ///             Subfield::new('a', "def")?,
    ///             Subfield::new('a', "hij")?,
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
            .filter(|x| x.code == code)
            .map(|x| &x.value)
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
    /// use pica::{Error, Field, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("003@")?,
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     );
    ///
    ///     assert_eq!(field.validate().is_ok(), true);
    ///
    ///     let field = Field::new(
    ///         Tag::new("003@")?,
    ///         None,
    ///         vec![
    ///             Subfield::new('0', "234567890X")?,
    ///             Subfield::new('1', vec![0, 159])?,
    ///             Subfield::new('2', "123456789X")?,
    ///         ],
    ///     );
    ///
    ///     assert_eq!(field.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<()> {
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
    /// use pica::{Field, Subfield, WriterBuilder, Occurrence, Tag};
    /// use std::error::Error;
    /// use tempfile::Builder;
    /// # use std::fs::read_to_string;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut tempfile = Builder::new().tempfile()?;
    ///     # let path = tempfile.path().to_owned();
    ///
    ///     let subfield = Subfield::new('0', "123456789X")?;
    ///     let occurrence = Occurrence::new("001")?;
    ///     let field = Field::new(Tag::new("012A")?, Some(occurrence), vec![subfield]);
    ///     
    ///     let mut writer = WriterBuilder::new().from_writer(tempfile);
    ///     field.write(&mut writer)?;
    ///     writer.finish()?;
    ///
    ///     # let result = read_to_string(path)?;
    ///     # assert_eq!(result, String::from("012A/001 \x1f0123456789X\x1e"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write(&self, writer: &mut dyn Write) -> crate::error::Result<()> {
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

impl fmt::Display for Field {
    /// Format the field in a human-readable format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, Field, Occurrence, Subfield, Tag};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         Tag::new("012A")?,
    ///         Some(Occurrence::new("01")?),
    ///         vec![
    ///             Subfield::new('0', "123456789X")?,
    ///             Subfield::new('a', "foo")?,
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
        // SAFETY: It's save because `Serialize` is only implemented for
        // `StringRecord` and not for `ByteRecord`.
        unsafe {
            state.serialize_field("tag", &self.tag.to_str_unchecked())?;
            if let Some(occurrence) = self.occurrence() {
                state.serialize_field(
                    "occurrence",
                    occurrence.to_str_unchecked(),
                )?;
            }
        }

        state.serialize_field("subfields", &self.subfields)?;
        state.end()
    }
}

#[inline]
pub(crate) fn parse_field(i: &[u8]) -> ParseResult<Field> {
    map(
        terminated(
            tuple((
                parse_tag,
                alt((
                    value(None, tag("/00")),
                    map(parse_occurrence, Some),
                    success(None),
                )),
                preceded(char(SP), many0(parse_subfield)),
            )),
            char(RS),
        ),
        |(tag, occurrence, subfields)| Field::new(tag, occurrence, subfields),
    )(i)
}

impl FromStr for Field {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match all_consuming(parse_field)(s.as_bytes()).finish() {
            Ok((_, field)) => Ok(field),
            Err(_) => Err(Error::InvalidField("invalid field!".to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FieldMatcher(TagMatcher, OccurrenceMatcher, SubfieldsMatcher);

impl FieldMatcher {
    pub fn is_match(&self, field: &Field, flags: &MatcherFlags) -> bool {
        field.tag() == &self.0
            && field.occurrence() == self.1
            && self.2.is_match(field.subfields(), flags)
    }
}

#[inline]
fn parse_field_matcher(i: &[u8]) -> ParseResult<FieldMatcher> {
    map(
        tuple((
            parse_tag_matcher,
            parse_occurrence_matcher,
            alt((
                preceded(char('.'), cut(parse_subfields_matcher_simple)),
                preceded(
                    ws(char('{')),
                    cut(terminated(parse_subfields_matcher, ws(char('}')))),
                ),
            )),
        )),
        |(tag, occurrence, subfields)| FieldMatcher(tag, occurrence, subfields),
    )(i)
}

impl FromStr for FieldMatcher {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match all_consuming(parse_field_matcher)(s.as_bytes()).finish() {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(Error::InvalidFieldMatcher(
                "invalid field matcher!".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::TestResult;

    #[test]
    fn test_field_from_str() -> TestResult {
        assert_eq!(
            Field::from_str("003@ \x1f0123456789X\x1e")?,
            Field::new(
                Tag::new("003@")?,
                None,
                vec![Subfield::new('0', "123456789X")?]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_field() -> TestResult {
        assert_eq!(
            parse_field(b"003@ \x1f0123456789X\x1e")?.1,
            Field::new(
                Tag::new("003@")?,
                None,
                vec![Subfield::new('0', "123456789X")?]
            )
        );

        assert_eq!(
            parse_field(b"012A/01 \x1f0abc\x1f0def\x1e")?.1,
            Field::new(
                Tag::new("012A")?,
                Some(Occurrence::new("01")?),
                vec![Subfield::new('0', "abc")?, Subfield::new('0', "def")?]
            )
        );

        assert_eq!(
            parse_field(b"012A/00 \x1f0abc\x1e")?.1,
            Field::new(
                Tag::new("012A")?,
                None,
                vec![Subfield::new('0', "abc")?]
            )
        );

        assert_eq!(
            parse_field(b"012A \x1e")?.1,
            Field::new(Tag::new("012A")?, None, vec![])
        );

        Ok(())
    }

    #[test]
    fn test_parse_field_matcher() -> TestResult {
        assert_eq!(
            parse_field_matcher(b"003@.0 == 'abc'")?.1,
            FieldMatcher(
                TagMatcher::Some(Tag::new("003@")?),
                OccurrenceMatcher::None,
                SubfieldsMatcher::from_str("0 == 'abc'")?,
            )
        );

        assert_eq!(
            parse_field_matcher(b"003@{0 == 'abc'}")?.1,
            FieldMatcher(
                TagMatcher::Some(Tag::new("003@")?),
                OccurrenceMatcher::None,
                SubfieldsMatcher::from_str("0 == 'abc'")?,
            )
        );

        assert_eq!(
            parse_field_matcher(b"003@{0? && 0 == 'abc'}")?.1,
            FieldMatcher(
                TagMatcher::Some(Tag::new("003@")?),
                OccurrenceMatcher::None,
                SubfieldsMatcher::from_str("0? && 0 == 'abc'")?,
            )
        );

        Ok(())
    }

    #[test]
    fn test_field_matcher_from_str() -> TestResult {
        assert_eq!(
            FieldMatcher::from_str("003@.0 == 'abc'")?,
            FieldMatcher(
                TagMatcher::Some(Tag::new("003@")?),
                OccurrenceMatcher::None,
                SubfieldsMatcher::from_str("0 == 'abc'")?
            )
        );

        assert!(FieldMatcher::from_str("003@.0 == 'abc").is_err());

        Ok(())
    }

    #[test]
    fn test_field_matcher_is_match() -> TestResult {
        let field = Field::from_str("003@ \x1f0123456789X\x1e")?;

        let matcher = FieldMatcher::from_str("003@.0 == '123456789X'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&field, &flags));

        let matcher = FieldMatcher::from_str("003@{ 0 == '123456789X' }")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&field, &flags));

        let field = Field::from_str("012A/01 \x1faabc\x1e")?;

        let matcher = FieldMatcher::from_str("012A/*.a == 'abc'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&field, &flags));

        let matcher = FieldMatcher::from_str("012A/*{a == 'abc'}")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&field, &flags));

        let matcher = FieldMatcher::from_str("012A/01.a == 'abc'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&field, &flags));

        let matcher = FieldMatcher::from_str("012A/01{a == 'abc'}")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&field, &flags));

        let matcher = FieldMatcher::from_str("012A.a == 'abc'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&field, &flags));

        let matcher = FieldMatcher::from_str("012A{a == 'abc'}")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&field, &flags));

        Ok(())
    }
}
