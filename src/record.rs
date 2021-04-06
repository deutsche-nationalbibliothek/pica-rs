//! This modules contains functions to parse PICA+ records.
//!
//! # Grammar
//!
//! ```text
//! <record>           :: <field>+ NL?
//! <field>            ::= <field-tag> <field-occurrence>? (SP <subfield>*)? RS
//! <field-tag>        ::= [0-2] [0-9]{2} ([A-Z] | '@')
//! <field-occurrence> ::= '/' [0-9]{2,3}
//! <subfield>         ::= US <subfield-code> <subfield-value>?
//! <subfield-code>    ::= [A-Za-z0-9]
//! <subfield-value>   ::= [^\x1e\x1f]
//!
//! <sp> ::= '\x20'
//! <us> ::= '\x1f'
//! <rs> ::= '\x1e'
//! ```

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, one_of, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, recognize, success};
use nom::multi::{many0, many1, many_m_n};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::Err;

use bstr::BString;
use regex::bytes::Regex;
use std::cmp::PartialEq;
use std::fmt;
use std::io::{self, Write};
use std::ops::Deref;
use std::result::Result as StdResult;

use crate::error::{Error, Result};
use crate::{Path, Writer};

lazy_static! {
    static ref FIELD_TAG_RE: Regex =
        Regex::new("^[0-2][0-9]{2}[A-Z@]$").unwrap();
}

/// A PICA+ occurrence.
#[derive(Clone, Debug, PartialEq)]
pub struct Occurrence(pub(crate) BString);

impl Deref for Occurrence {
    type Target = BString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<&str> for Occurrence {
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

impl PartialEq<str> for Occurrence {
    fn eq(&self, other: &str) -> bool {
        self.0 == other.as_bytes()
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
    ///
    /// assert!(Subfield::new('0', "12283643X").is_ok());
    /// assert!(Subfield::new('!', "12283643X").is_err());
    /// assert!(Subfield::new('a', "123\x1f34").is_err());
    /// assert!(Subfield::new('a', "123\x1e34").is_err());
    /// ```
    pub fn new<S>(code: char, value: S) -> Result<Subfield>
    where
        S: Into<BString>,
    {
        let value: BString = value.into();

        if !code.is_ascii_alphanumeric() {
            return Err(Error::InvalidSubfield(format!(
                "Invalid subfield code '{}'",
                code
            )));
        }

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
        Self {
            code,
            value: value.into(),
        }
    }

    /// Returns the subfield code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('a', "1234")?;
    ///     assert_eq!(subfield.code(), 'a');
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> char {
        self.code
    }

    /// Returns the subfield value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('a', "1234")?;
    ///     assert_eq!(subfield.value(), "1234");
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
    /// use pica::{Subfield, WriterBuilder};
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
    ///     writer.flush()?;
    ///
    ///     # let result = read_to_string(path)?;
    ///     # assert_eq!(result, String::from("\x1f0123456789X"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write<W: io::Write>(
        &self,
        writer: &mut Writer<W>,
    ) -> crate::error::Result<()> {
        write!(writer, "\x1f{}{}", self.code, self.value)?;
        Ok(())
    }
}

/// A PICA+ field, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq)]
pub struct Field {
    pub(crate) tag: BString,
    pub(crate) occurrence: Option<Occurrence>,
    pub(crate) subfields: Vec<Subfield>,
}

impl Deref for Field {
    type Target = Vec<Subfield>;

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

    /// Retrun the fields tag
    ///
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

    /// Retrun the field's occurrence
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Field, Occurrence};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let field = Field::new("003@", Some(Occurrence::new("00")?), vec![])?;
    ///     assert_eq!(field.occurrence().unwrap(), "00");
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
    ///     writer.flush()?;
    ///
    ///     # let result = read_to_string(path)?;
    ///     # assert_eq!(result, String::from("012A/001 \x1f0123456789X\x1e"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write<W: io::Write>(
        &self,
        writer: &mut Writer<W>,
    ) -> crate::error::Result<()> {
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

/// A PICA+ record, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq)]
pub struct ByteRecord {
    pub(crate) raw_data: Option<Vec<u8>>,
    pub(crate) fields: Vec<Field>,
}

impl Deref for ByteRecord {
    type Target = Vec<Field>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
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

    /// Returns `true` if a field with the specified `tag` exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{ByteRecord, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let data = include_str!("../tests/data/12283643X.dat");
    ///     let record = ByteRecord::from_bytes(data.as_bytes())?;
    ///
    ///     assert_eq!(record.contains_tag("003@"), true);
    ///     assert_eq!(record.contains_tag("012A"), false);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn contains_tag<S>(&self, tag: S) -> bool
    where
        S: Into<BString>,
    {
        let tag = tag.into();

        self.iter().any(|x| x.tag() == &tag)
    }

    /// # Example
    ///
    /// ```rust
    /// use pica::{Path, StringRecord};
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = StringRecord::from_bytes(
    ///         "012A \x1fa123\x1fa456\x1e012A/00 \x1fa789\x1e",
    ///     )?;
    ///
    ///     assert_eq!(
    ///         record.path(&Path::from_str("012A/*.a")?),
    ///         vec!["123", "456", "789"]
    ///     );
    ///
    ///     assert_eq!(record.path(&Path::from_str("012A.a")?), vec!["123", "456"]);
    ///     assert_eq!(record.path(&Path::from_str("012A/00.a")?), vec!["789"]);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn path(&self, path: &Path) -> Vec<&BString> {
        self.fields
            .iter()
            .filter(|field| {
                field.tag == path.tag
                    && field.occurrence == path.occurrence
                    && field.contains_code(path.code)
            })
            .flat_map(|field| field.get(path.code).unwrap())
            .map(|subfield| subfield.value())
            .collect()
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
    ///     writer.flush()?;
    ///
    ///     # let result = read_to_string(path)?;
    ///     # assert_eq!(result, String::from(
    ///     #     "012A/001 \x1f0123456789X\x1e012A/002 \x1f0123456789X\x1e\n"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write<W: io::Write>(
        &self,
        writer: &mut Writer<W>,
    ) -> crate::error::Result<()> {
        for field in &self.fields {
            field.write(writer)?;
        }

        writer.write_all(b"\n")?;
        Ok(())
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
        Ok(StringRecord::from_byte_record(record)?)
    }
}

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, PartialEq)]
pub struct ParsePicaError {
    pub message: String,
    pub data: Vec<u8>,
}

impl std::error::Error for ParsePicaError {}

impl fmt::Display for ParsePicaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}

pub(crate) type ParseResult<'a, O> = StdResult<(&'a [u8], O), Err<()>>;

/// Parses a PICA+ subfield code.
pub(crate) fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    map(satisfy(|c| c.is_ascii_alphanumeric()), char::from)(i)
}

/// Parses a PICA+ subfield value.
fn parse_subfield_value(i: &[u8]) -> ParseResult<BString> {
    map(recognize(many0(is_not("\x1E\x1F"))), BString::from)(i)
}

// Parses a PICA+ subfield.
fn parse_subfield(i: &[u8]) -> ParseResult<Subfield> {
    map(
        preceded(
            char('\x1f'),
            cut(pair(parse_subfield_code, parse_subfield_value)),
        ),
        |(code, value)| Subfield::from_unchecked(code, value),
    )(i)
}

/// Parses a PICA+ field occurrence.
pub(crate) fn parse_field_occurrence(i: &[u8]) -> ParseResult<Occurrence> {
    map(
        preceded(
            tag(b"/"),
            cut(recognize(many_m_n(2, 3, one_of("0123456789")))),
        ),
        Occurrence::from_unchecked,
    )(i)
}

/// Parses a PICA+ Field tag.
pub(crate) fn parse_field_tag(i: &[u8]) -> ParseResult<BString> {
    map(
        recognize(tuple((
            one_of("012"),
            one_of("0123456789"),
            one_of("0123456789"),
            satisfy(|c| c.is_ascii_uppercase() || c == '@'),
        ))),
        BString::from,
    )(i)
}

/// Parses a PICA+ field.
fn parse_field(i: &[u8]) -> ParseResult<Field> {
    map(
        terminated(
            tuple((
                parse_field_tag,
                alt((map(parse_field_occurrence, Some), success(None))),
                preceded(char(' '), many0(parse_subfield)),
            )),
            char('\x1e'),
        ),
        |(tag, occurrence, subfields)| Field {
            tag,
            occurrence,
            subfields,
        },
    )(i)
}

/// Parse a PICA+ record.
fn parse_fields(i: &[u8]) -> ParseResult<Vec<Field>> {
    all_consuming(terminated(many1(parse_field), opt(char('\n'))))(i)
}

#[cfg(test)]
mod test {
    use super::*;

    type TestResult = StdResult<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_parse_subfield_code() {
        assert_eq!(parse_subfield_code(b"a").unwrap().1, 'a');
        assert!(parse_subfield_code(b"!").is_err());
    }

    #[test]
    fn test_parse_subfield_value() {
        assert_eq!(
            parse_subfield_value(b"foobarbaz").unwrap().1,
            BString::from("foobarbaz")
        );
        assert_eq!(
            parse_subfield_value(b"123\x1fdef").unwrap().1,
            BString::from("123")
        );
        assert_eq!(
            parse_subfield_value(b"123\x1edef").unwrap().1,
            BString::from("123")
        );
    }

    #[test]
    fn test_parse_subfield() -> TestResult {
        assert_eq!(
            parse_subfield(b"\x1fa123456789").unwrap().1,
            Subfield::new('a', "123456789")?
        );

        Ok(())
    }

    #[test]
    fn test_parse_field_occurrence() -> TestResult {
        assert_eq!(
            parse_field_occurrence(b"/01").unwrap().1,
            Occurrence::new("01")?
        );

        assert_eq!(
            parse_field_occurrence(b"/001").unwrap().1,
            Occurrence::new("001")?
        );

        assert!(parse_field_occurrence(b"/0A").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_field_tag() {
        assert_eq!(parse_field_tag(b"003@").unwrap().1, BString::from("003@"));
        assert!(parse_field_tag(b"303@").is_err());
        assert!(parse_field_tag(b"0A3@").is_err());
        assert!(parse_field_tag(b"00A@").is_err());
        assert!(parse_field_tag(b"0000").is_err());
    }

    #[test]
    fn test_parse_field() -> TestResult {
        assert_eq!(
            parse_field(b"012A/00 \x1fa123\x1fb456\x1fc789\x1e")
                .unwrap()
                .1,
            Field::new(
                "012A",
                Some(Occurrence::new("00")?),
                vec![
                    Subfield::new('a', "123")?,
                    Subfield::new('b', "456")?,
                    Subfield::new('c', "789")?,
                ]
            )?
        );

        Ok(())
    }

    #[test]
    fn test_parse_fields() -> TestResult {
        let record_str =
            b"012A \x1fc789\x1e012A/00 \x1fa123\x1e012A/01 \x1fb456\x1e";
        assert_eq!(
            parse_fields(record_str).unwrap().1,
            vec![
                Field::new("012A", None, vec![Subfield::new('c', "789")?])?,
                Field::new(
                    "012A",
                    Some(Occurrence::new("00")?),
                    vec![Subfield::new('a', "123")?]
                )?,
                Field::new(
                    "012A",
                    Some(Occurrence::new("01")?),
                    vec![Subfield::new('b', "456")?]
                )?,
            ]
        );

        assert_eq!(
            parse_fields(b"012A \x1fa123\x1e\n").unwrap().1,
            vec![Field::new("012A", None, vec![Subfield::new('a', "123")?])?]
        );

        Ok(())
    }
}
