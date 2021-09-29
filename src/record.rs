use crate::error::{Error, Result};
use crate::parser::{parse_fields, ParsePicaError};
use crate::select::{Outcome, Selector};
use crate::Path;

use bstr::{BString, ByteSlice};
use regex::bytes::Regex;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::cmp::PartialEq;
use std::fmt;
use std::io::Write;
use std::ops::Deref;
use std::result::Result as StdResult;

lazy_static! {
    pub(crate) static ref FIELD_TAG_RE: Regex =
        Regex::new("^[0-2][0-9]{2}[A-Z@]$").unwrap();
}

/// A PICA+ subfield, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq)]
pub struct Subfield {
    pub(crate) code: char,
    pub(crate) value: BString,
}

impl Subfield {
    /// Creates a new `Subfield`
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    /// use std::error::Error;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     assert!(Subfield::new('0', "12283643X").is_ok());
    ///     assert!(Subfield::new('!', "12283643X").is_err());
    ///     assert!(Subfield::new('a', "123\x1f34").is_err());
    ///     assert!(Subfield::new('a', "123\x1e34").is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S>(code: char, value: S) -> Result<Subfield>
    where
        S: Into<BString>,
    {
        if !code.is_ascii_alphanumeric() {
            return Err(Error::InvalidSubfield(format!(
                "Invalid subfield code '{}'",
                code
            )));
        }

        let value: BString = value.into();
        if value.contains(&b'\x1e') || value.contains(&b'\x1f') {
            return Err(Error::InvalidSubfield(
                "Invalid subfield value.".to_string(),
            ));
        }

        Ok(Subfield { code, value })
    }

    /// Creates a new `Subfield` without checking for valid code or value.
    #[inline]
    pub(crate) fn from_unchecked<S>(code: char, value: S) -> Subfield
    where
        S: Into<BString>,
    {
        Subfield {
            code,
            value: value.into(),
        }
    }

    /// Get a reference to the subfield's code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "12283643X")?;
    ///     assert_eq!(subfield.code(), '0');
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> char {
        self.code
    }

    /// Get a reference to the subfield's value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "12283643X")?;
    ///     assert_eq!(subfield.value(), "12283643X");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &BString {
        &self.value
    }

    /// Returns `true` if the subfield value is valid UTF-8 byte sequence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "123456789X")?;
    ///     assert_eq!(subfield.validate().is_ok(), true);
    ///
    ///     let subfield = Subfield::new('0', vec![0, 159])?;
    ///     assert_eq!(subfield.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.value.is_ascii() {
            return Ok(());
        }

        if let Err(e) = std::str::from_utf8(&self.value) {
            return Err(Error::Utf8Error(e));
        }

        Ok(())
    }

    /// Write the field into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{PicaWriter, Subfield, WriterBuilder};
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
    ///     let mut writer = WriterBuilder::new().from_writer(tempfile);
    ///     subfield.write(&mut writer)?;
    ///     writer.finish()?;
    ///
    ///     # let result = read_to_string(path)?;
    ///     # assert_eq!(result, String::from("\x1f0123456789X"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> crate::error::Result<()> {
        write!(writer, "\x1f{}{}", self.code, self.value)?;
        Ok(())
    }
}

impl fmt::Display for Subfield {
    /// Format the subfield in a human-readable format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "123456789X")?;
    ///     assert_eq!(format!("{}", subfield), "$0123456789X");
    ///
    ///     Ok(())
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        write!(f, "${}{}", self.code, self.value)
    }
}

impl Serialize for Subfield {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Subfield", 2)?;
        state.serialize_field("name", &self.code)?;
        // SAFETY: It's save because `Serialize` is only implemented for
        // `StringRecord` and not for `ByteRecord`.
        unsafe {
            state.serialize_field("value", &self.value.to_str_unchecked())?;
        }
        state.end()
    }
}

/// A PICA+ occurrence.
#[derive(Debug, PartialEq, Clone)]
pub struct Occurrence(pub(crate) BString);

impl Deref for Occurrence {
    type Target = BString;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<str> for Occurrence {
    fn eq(&self, other: &str) -> bool {
        self.0 == other.as_bytes()
    }
}

impl PartialEq<&str> for Occurrence {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl fmt::Display for Occurrence {
    /// Format the field in a human-readable format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, Occurrence};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let occurrence = Occurrence::new("01")?;
    ///     assert_eq!(format!("{}", occurrence), "/01");
    ///
    ///     Ok(())
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        write!(f, "/{}", self.0)?;

        Ok(())
    }
}

impl Occurrence {
    /// Creates a new `Occurrence`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bstr::BString;
    /// use pica::Occurrence;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let occurrence = Occurrence::new("00")?;
    ///     assert_eq!(occurrence, "00");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S>(occurrence: S) -> Result<Occurrence>
    where
        S: Into<BString>,
    {
        let occurrence = occurrence.into();

        if occurrence.len() < 2 || occurrence.len() > 3 {
            return Err(Error::InvalidOccurrence(
                "length < 2 || length > 3".to_string(),
            ));
        }

        if !occurrence.iter().all(u8::is_ascii_digit) {
            return Err(Error::InvalidOccurrence(format!(
                "Invalid occurrence '{}'",
                occurrence
            )));
        }

        Ok(Occurrence(occurrence))
    }

    /// Creates a new `Occurrence` without checking the input.
    #[inline]
    pub(crate) fn from_unchecked<S>(occurrence: S) -> Occurrence
    where
        S: Into<BString>,
    {
        Self(occurrence.into())
    }
}

/// A PICA+ field, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq)]
pub struct Field {
    pub(crate) tag: BString,
    pub(crate) occurrence: Option<Occurrence>,
    pub(crate) subfields: Vec<Subfield>,
}

impl Field {
    /// Creates a new `Field`
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field =
    ///         Field::new("003@", None, vec![Subfield::new('0', "123456789X")?])?;
    ///     assert_eq!(field.tag(), "003@");
    ///     assert_eq!(field.len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S>(
        tag: S,
        occurrence: Option<Occurrence>,
        subfields: Vec<Subfield>,
    ) -> Result<Field>
    where
        S: Into<BString>,
    {
        let tag = tag.into();

        if !FIELD_TAG_RE.is_match(tag.as_slice()) {
            return Err(Error::InvalidField("Invalid field tag.".to_string()));
        }

        Ok(Field {
            tag,
            occurrence,
            subfields,
        })
    }

    /// Get a reference to the field's tag.
    /// # Example
    ///
    /// ```rust
    /// use pica::Field;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new("003@", None, vec![])?;
    ///     assert_eq!(field.tag(), "003@");
    ///     Ok(())
    /// }
    /// ```
    pub fn tag(&self) -> &BString {
        &self.tag
    }

    /// Get a reference to the field's occurrence.
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Occurrence};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new("012A", Some(Occurrence::new("00")?), vec![])?;
    ///     assert_eq!(field.occurrence(), Some(&Occurrence::new("00")?));
    ///     Ok(())
    /// }
    /// ```
    pub fn occurrence(&self) -> Option<&Occurrence> {
        self.occurrence.as_ref()
    }

    /// Returns `true` if the `Field` contains a `Subfield` with the specified
    ///  code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field =
    ///         Field::new("003@", None, vec![Subfield::new('0', "123456789X")?])?;
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
    /// use pica::{Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field =
    ///         Field::new("003@", None, vec![Subfield::new('0', "123456789X")?])?;
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
    /// use pica::{Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         "003@",
    ///         None,
    ///         vec![
    ///             Subfield::new('a', "abc")?,
    ///             Subfield::new('a', "def")?,
    ///             Subfield::new('a', "hij")?,
    ///         ],
    ///     )?;
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

    /// Returns the all subfield value for the subfield code
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         "003@",
    ///         None,
    ///         vec![
    ///             Subfield::new('a', "abc")?,
    ///             Subfield::new('a', "def")?,
    ///             Subfield::new('a', "hij")?,
    ///         ],
    ///     )?;
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
    /// use pica::{Error, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field =
    ///         Field::new("003@", None, vec![Subfield::new('0', "123456789X")?])?;
    ///     assert_eq!(field.validate().is_ok(), true);
    ///
    ///     let field = Field::new(
    ///         "003@",
    ///         None,
    ///         vec![
    ///             Subfield::new('0', "234567890X")?,
    ///             Subfield::new('1', vec![0, 159])?,
    ///             Subfield::new('2', "123456789X")?,
    ///         ],
    ///     )?;
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
    /// use pica::{Field, Subfield, WriterBuilder, Occurrence};
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
    ///     let field = Field::new("012A", Some(occurrence), vec![subfield])?;
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
            write!(writer, "/{}", occurrence.0)?;
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
    /// use pica::{Error, Field, Occurrence, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new(
    ///         "012A",
    ///         Some(Occurrence::new("01")?),
    ///         vec![
    ///             Subfield::new('0', "123456789X")?,
    ///             Subfield::new('a', "foo")?,
    ///         ],
    ///     )?;
    ///
    ///     assert_eq!(format!("{}", field), "012A/01 $0123456789X$afoo");
    ///
    ///     Ok(())
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        write!(f, "{}", self.tag)?;

        if let Some(ref occurrence) = self.occurrence {
            write!(f, "/{}", occurrence.0)?;
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

impl Deref for Field {
    type Target = Vec<Subfield>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.subfields
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
            state.serialize_field("name", &self.tag.to_str_unchecked())?;
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

/// A PICA+ record, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq)]
pub struct ByteRecord {
    pub(crate) raw_data: Option<Vec<u8>>,
    pub(crate) fields: Vec<Field>,
}

impl ByteRecord {
    /// Creates a new `ByteRecord`
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = ByteRecord::new(vec![Field::new(
    ///         "003@",
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     )?]);
    ///
    ///     assert_eq!(record.len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(fields: Vec<Field>) -> ByteRecord {
        ByteRecord {
            fields,
            raw_data: None,
        }
    }

    /// Creates a new ByteRecord from a byte vector.
    ///
    /// Parses the given byte sequence and return the corresponding
    /// `ByteRecord`. If an parse error occurs an `ParsePicaError` will be
    /// returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
    ///     assert_eq!(record.len(), 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes<T>(data: T) -> StdResult<ByteRecord, ParsePicaError>
    where
        T: Into<Vec<u8>>,
    {
        let data = data.into();

        let fields = parse_fields(&data)
            .map_err(|_| ParsePicaError {
                message: "Invalid record.".to_string(),
                data: data.clone(),
            })
            .map(|(_, record)| record)?;

        Ok(ByteRecord {
            raw_data: Some(data),
            fields,
        })
    }

    /// Returns `true` if no fields contains invalid subfield values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Error, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = ByteRecord::new(vec![Field::new(
    ///         "003@",
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     )?]);
    ///     assert_eq!(record.validate().is_ok(), true);
    ///
    ///     let record = ByteRecord::new(vec![Field::new(
    ///         "003@",
    ///         None,
    ///         vec![
    ///             Subfield::new('0', "234567890X")?,
    ///             Subfield::new('1', vec![0, 159])?,
    ///             Subfield::new('2', "123456789X")?,
    ///         ],
    ///     )?]);
    ///     assert_eq!(record.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<()> {
        for field in &self.fields {
            field.validate()?;
        }

        Ok(())
    }

    /// Write the field into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Subfield, WriterBuilder, Occurrence, ByteRecord};
    /// use std::error::Error;
    /// use tempfile::Builder;
    /// # use std::fs::read_to_string;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut tempfile = Builder::new().tempfile()?;
    ///     # let path = tempfile.path().to_owned();
    ///
    ///     let record = ByteRecord::new(vec![
    ///         Field::new("012A", Some(Occurrence::new("001")?), vec![
    ///             Subfield::new('0', "123456789X")?,
    ///         ])?,
    ///         Field::new("012A", Some(Occurrence::new("002")?), vec![
    ///             Subfield::new('0', "123456789X")?,
    ///         ])?,
    ///     ]);
    ///
    ///     let mut writer = WriterBuilder::new().from_writer(tempfile);
    ///     record.write(&mut writer)?;
    ///     writer.finish()?;
    ///
    ///     # let result = read_to_string(path)?;
    ///     # assert_eq!(result, String::from(
    ///     #     "012A/001 \x1f0123456789X\x1e012A/002 \x1f0123456789X\x1e\n"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write(&self, writer: &mut dyn Write) -> crate::error::Result<()> {
        for field in &self.fields {
            field.write(writer)?;
        }

        writer.write_all(b"\n")?;
        Ok(())
    }

    /// Returns the first field matching the given tag
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
    ///     assert_eq!(
    ///         record.first("003@"),
    ///         Some(&Field::new(
    ///             "003@",
    ///             None,
    ///             vec![Subfield::new('0', "123456789X")?]
    ///         )?)
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn first(&self, tag: &str) -> Option<&Field> {
        self.iter().find(|field| field.tag == tag)
    }

    /// Returns all fields matching the given tag
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes("012A \x1fa123\x1e012A \x1fa456\x1e")?;
    ///
    ///     assert_eq!(
    ///         record.all("012A"),
    ///         Some(vec![
    ///             &Field::new("012A", None, vec![Subfield::new('a', "123")?])?,
    ///             &Field::new("012A", None, vec![Subfield::new('a', "456")?])?,
    ///         ])
    ///     );
    ///
    ///     assert_eq!(record.all("012B"), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn all(&self, tag: &str) -> Option<Vec<&Field>> {
        let result = self
            .iter()
            .filter(|field| field.tag == tag)
            .collect::<Vec<&Field>>();

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }

    /// Returns all subfield values of a given path.
    pub fn path(&self, path: &Path) -> Vec<&BString> {
        self.fields
            .iter()
            .filter(|field| {
                field.tag == path.tag
                    && field.occurrence == path.occurrence
                    && path.codes.iter().any(|x| field.contains_code(*x))
            })
            .flat_map(|field| {
                path.codes
                    .iter()
                    .flat_map(move |code| field.get(*code).unwrap_or_default())
            })
            .map(|subfield| subfield.value())
            .collect()
    }

    pub fn select(&self, selector: &Selector, ignore_case: bool) -> Outcome {
        match selector {
            Selector::Value(value) => {
                Outcome::from_values(vec![BString::from(value.as_bytes())])
            }
            Selector::Field(selector) => {
                let result = self
                    .iter()
                    .filter(|field| selector.tag == field.tag)
                    .filter(|field| field.occurrence == selector.occurrence)
                    .filter(|field| {
                        if let Some(filter) = &selector.filter {
                            filter.matches(field, ignore_case)
                        } else {
                            true
                        }
                    })
                    .map(|field| &field.subfields)
                    .map(|subfields| {
                        selector
                            .subfields
                            .iter()
                            .map(|code| {
                                subfields
                                    .iter()
                                    .filter(|subfield| subfield.code == *code)
                                    .map(|subfield| {
                                        vec![subfield.value().to_owned()]
                                    })
                                    .collect::<Vec<Vec<BString>>>()
                            })
                            .map(|x| {
                                if x.is_empty() {
                                    Outcome::one()
                                } else {
                                    Outcome(x)
                                }
                            })
                            .fold(Outcome::default(), |acc, x| acc * x)
                    })
                    .fold(Outcome::default(), |acc, x| acc + x);

                if result.is_empty() {
                    let mut values: Vec<BString> =
                        Vec::with_capacity(selector.subfields.len());
                    for _ in 0..selector.subfields.len() {
                        values.push(BString::from(""));
                    }

                    Outcome::from_values(values)
                } else {
                    result
                }
            }
        }
    }
}

impl fmt::Display for ByteRecord {
    /// Format the subfield in a human-readable format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Error};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = ByteRecord::from_bytes(
    ///         "003@ \x1f0123456789X\x1e012A/01 \x1fa123\x1e",
    ///     )?;
    ///     assert_eq!(format!("{}", record), "003@ $0123456789X\n012A/01 $a123");
    ///
    ///     Ok(())
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        let fields = self
            .fields
            .iter()
            .map(|f| format!("{}", f))
            .collect::<Vec<_>>()
            .join("\n");

        write!(f, "{}", fields)
    }
}

impl Deref for ByteRecord {
    type Target = Vec<Field>;

    /// Dereferences the value
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

/// A PICA+ record, that guarantees valid UTF-8 data.
#[derive(Debug, PartialEq)]
pub struct StringRecord(ByteRecord);

impl Deref for StringRecord {
    type Target = ByteRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl StringRecord {
    /// Creates a new `StringRecord` from a `ByteRecord`
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, StringRecord};
    /// use std::error::Error;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e".to_vec())?;
    ///     assert!(StringRecord::from_byte_record(record).is_ok());
    ///
    ///     let record =
    ///         ByteRecord::from_bytes(b"003@ \x1ffoo\xffbar\x1e".to_vec())?;
    ///     assert!(StringRecord::from_byte_record(record).is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_byte_record(record: ByteRecord) -> Result<StringRecord> {
        match record.validate() {
            Ok(()) => Ok(StringRecord(record)),
            Err(e) => Err(e),
        }
    }

    /// Creates a new `StringRecord` from a bytes vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, StringRecord};
    /// use std::error::Error;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let result = StringRecord::from_bytes("003@ \x1f0123456789X\x1e");
    ///     assert!(result.is_ok());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes<T: Into<Vec<u8>>>(data: T) -> Result<StringRecord> {
        let record = ByteRecord::from_bytes(data.into())?;
        StringRecord::from_byte_record(record)
    }
}

impl fmt::Display for StringRecord {
    /// Format the subfield in a human-readable format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, StringRecord};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = StringRecord::from_bytes(
    ///         "003@ \x1f0123456789X\x1e012A/01 \x1fa123\x1e",
    ///     )?;
    ///     assert_eq!(format!("{}", record), "003@ $0123456789X\n012A/01 $a123");
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Serialize for StringRecord {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Record", 1)?;
        state.serialize_field("fields", &self.fields)?;
        state.end()
    }
}
