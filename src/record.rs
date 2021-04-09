use crate::error::{Error, Result};
use crate::parser::{parse_fields, ParsePicaError};
use crate::select::{Outcome, Selector};
use crate::Path;

use bstr::BString;
use regex::bytes::Regex;
use serde::Serialize;
use std::fmt;
use std::ops::Deref;
use std::result::Result as StdResult;

lazy_static! {
    pub(crate) static ref FIELD_TAG_RE: Regex =
        Regex::new("^[0-2][0-9]{2}[A-Z@]$").unwrap();
}

/// A PICA+ subfield, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq, Serialize)]
pub struct Subfield {
    #[serde(rename(serialize = "name"))]
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
    ///     assert_eq!(subfield.code(), &'0');
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> &char {
        &self.code
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
    ///     assert_eq!(format!("{}", subfield), "$0 123456789X");
    ///
    ///     Ok(())
    /// }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        write!(f, "${} {}", self.code, self.value)
    }
}

/// A PICA+ occurrence.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Occurrence(pub(crate) BString);

impl Deref for Occurrence {
    type Target = BString;

    /// Dereferences the value
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

/// A PICA+ field, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq, Serialize)]
pub struct Field {
    #[serde(rename(serialize = "name"))]
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
    ///     assert_eq!(format!("{}", field), "012A/01 $0 123456789X $a foo");
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
                .join(" ");

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

/// A PICA+ record, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq, Serialize)]
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

    /// Returns all subfield values of a given path.
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

    /// Returns the record as an human readable string.
    pub fn pretty(&self) -> String {
        self.fields
            .iter()
            .map(|f| format!("{}", f))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn select(&self, selector: &Selector) -> Outcome {
        let result = self
            .iter()
            .filter(|field| selector.tag == field.tag)
            .filter(|field| selector.occurrence == field.occurrence)
            .filter(|field| {
                if let Some(filter) = &selector.filter {
                    filter.matches(&field)
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
                            .map(|subfield| vec![subfield.value().to_owned()])
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
    ///     assert_eq!(format!("{}", record), "003@ $0 123456789X\n012A/01 $a 123");
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
