use crate::error::Result;
use crate::matcher::{MatcherFlags, TagMatcher};
use crate::parser::{parse_fields, ParsePicaError};
use crate::select::{Outcome, Selector};
use crate::{Field, Path};

use bstr::BString;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::cmp::PartialEq;
use std::fmt;
use std::io::Write;
use std::ops::Deref;
use std::result::Result as StdResult;

/// A PICA+ record, that may contian invalid UTF-8 data.
#[derive(Debug, PartialEq, Eq)]
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
    /// use pica_core::Tag;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = ByteRecord::new(vec![Field::new(
    ///         Tag::from_str("003@")?,
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     )]);
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
    /// use pica_core::Tag;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = ByteRecord::new(vec![Field::new(
    ///         Tag::from_str("003@")?,
    ///         None,
    ///         vec![Subfield::new('0', "123456789X")?],
    ///     )]);
    ///     assert_eq!(record.validate().is_ok(), true);
    ///
    ///     let record = ByteRecord::new(vec![Field::new(
    ///         Tag::from_str("003@")?,
    ///         None,
    ///         vec![
    ///             Subfield::new('0', "234567890X")?,
    ///             Subfield::new('1', vec![0, 159])?,
    ///             Subfield::new('2', "123456789X")?,
    ///         ],
    ///     )]);
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
    /// use pica::{ByteRecord, Field, Subfield, WriterBuilder};
    /// use pica_core::Tag;
    /// use pica_core::Occurrence;
    /// use std::error::Error;
    /// use tempfile::Builder;
    /// use std::str::FromStr;
    /// # use std::fs::read_to_string;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut tempfile = Builder::new().tempfile()?;
    ///     # let path = tempfile.path().to_owned();
    ///
    ///     let record = ByteRecord::new(vec![
    ///         Field::new(
    ///             Tag::from_str("012A")?,
    ///             Some(Occurrence::from_str("/001")?),
    ///             vec![Subfield::new('0', "123456789X")?],
    ///         ),
    ///         Field::new(
    ///             Tag::from_str("012A")?,
    ///             Some(Occurrence::from_str("/002")?),
    ///             vec![Subfield::new('0', "123456789X")?],
    ///         ),
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
    /// use pica_core::Tag;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
    ///     assert_eq!(
    ///         record.first("003@"),
    ///         Some(&Field::new(
    ///             Tag::from_str("003@")?,
    ///             None,
    ///             vec![Subfield::new('0', "123456789X")?]
    ///         ))
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
    /// use pica_core::Tag;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let record =
    ///         ByteRecord::from_bytes("012A \x1fa123\x1e012A \x1fa456\x1e")?;
    ///
    ///     assert_eq!(
    ///         record.all("012A"),
    ///         Some(vec![
    ///             &Field::new(
    ///                 Tag::from_str("012A")?,
    ///                 None,
    ///                 vec![Subfield::new('a', "123")?]
    ///             ),
    ///             &Field::new(
    ///                 Tag::from_str("012A")?,
    ///                 None,
    ///                 vec![Subfield::new('a', "456")?]
    ///             ),
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
                path.tag.is_match(field.tag())
                    && path.occurrence.is_match(field.occurrence())
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
                    .filter(|field| selector.tag.is_match(field.tag()))
                    .filter(|field| {
                        selector.occurrence.is_match(field.occurrence())
                    })
                    .filter(|field| {
                        if let Some(filter) = &selector.filter {
                            filter.is_match(
                                field,
                                &MatcherFlags {
                                    ignore_case,
                                    strsim_threshold: 0.0,
                                },
                            )
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

    /// Reduce the record to the given fields.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::TagMatcher;
    /// use pica::{ByteRecord, Field, Subfield};
    /// use pica_core::Tag;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut record =
    ///         ByteRecord::from_bytes("012A \x1fa123\x1e013A \x1fa456\x1e")?;
    ///     record.reduce(&[TagMatcher::new("003@")?, TagMatcher::new("012A")?]);
    ///
    ///     assert_eq!(
    ///         record,
    ///         ByteRecord::new(vec![Field::new(
    ///             Tag::from_str("012A")?,
    ///             None,
    ///             vec![Subfield::new('a', "123")?],
    ///         )])
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn reduce(&mut self, matchers: &[TagMatcher]) {
        if !matchers.is_empty() {
            self.raw_data = None;
            self.fields = self
                .fields
                .clone()
                .into_iter()
                .filter(|field| matchers.iter().any(|m| m.is_match(&field.tag)))
                .collect();
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
#[derive(Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::str::FromStr;

    use pica_core::{Occurrence, Tag};

    use crate::test::TestResult;
    use crate::Subfield;

    #[test]
    fn test_field_new() -> TestResult {
        assert_eq!(
            Field::new(
                Tag::from_str("003@")?,
                None,
                vec![Subfield::new('0', "123456789X")?]
            ),
            Field {
                tag: Tag::from_str("003@")?,
                occurrence: None,
                subfields: vec![Subfield::new('0', "123456789X")?]
            }
        );

        Ok(())
    }

    #[test]
    fn test_field_tag() -> TestResult {
        let field = Field::new(
            Tag::from_str("003@")?,
            None,
            vec![Subfield::new('0', "123456789X")?],
        );

        assert_eq!(field.tag(), &Tag::from_str("003@")?);
        Ok(())
    }

    #[test]
    fn test_field_occurrence() -> TestResult {
        let field = Field::new(
            Tag::from_str("003@")?,
            None,
            vec![Subfield::new('0', "123456789X")?],
        );

        assert_eq!(field.occurrence(), None);

        let field = Field::new(
            Tag::from_str("003@")?,
            Some(Occurrence::from_str("/01")?),
            vec![Subfield::new('0', "123456789X")?],
        );

        assert_eq!(field.occurrence(), Some(&Occurrence::from_str("/01")?));

        Ok(())
    }

    #[test]
    fn test_field_contains_code() -> TestResult {
        let field = Field::new(
            Tag::from_str("003@")?,
            None,
            vec![Subfield::new('0', "123456789X")?],
        );

        assert!(field.contains_code('0'));
        assert!(!field.contains_code('a'));

        Ok(())
    }

    #[test]
    fn test_field_get() -> TestResult {
        let field = Field::new(
            Tag::from_str("012A")?,
            None,
            vec![
                Subfield::new('a', "abc")?,
                Subfield::new('b', "def")?,
                Subfield::new('a', "hij")?,
            ],
        );

        assert_eq!(
            field.get('a'),
            Some(vec![
                &Subfield::new('a', "abc")?,
                &Subfield::new('a', "hij")?
            ])
        );
        assert_eq!(field.get('b'), Some(vec![&Subfield::new('b', "def")?]));
        assert_eq!(field.get('c'), None,);

        Ok(())
    }

    #[test]
    fn test_field_first() -> TestResult {
        let field = Field::new(
            Tag::from_str("012A")?,
            None,
            vec![
                Subfield::new('a', "abc")?,
                Subfield::new('b', "def")?,
                Subfield::new('a', "hij")?,
            ],
        );

        assert_eq!(field.first('a'), Some(&BString::from("abc")));
        assert_eq!(field.first('b'), Some(&BString::from("def")));
        assert_eq!(field.first('c'), None);

        Ok(())
    }

    #[test]
    fn test_field_all() -> TestResult {
        let field = Field::new(
            Tag::from_str("012A")?,
            None,
            vec![
                Subfield::new('a', "abc")?,
                Subfield::new('b', "def")?,
                Subfield::new('a', "hij")?,
            ],
        );

        assert_eq!(
            field.all('a'),
            Some(vec![&BString::from("abc"), &BString::from("hij")])
        );
        assert_eq!(field.all('b'), Some(vec![&BString::from("def")]));
        assert_eq!(field.all('c'), None);

        Ok(())
    }

    #[test]
    fn test_field_validate() -> TestResult {
        let field = Field::new(
            Tag::from_str("012A")?,
            None,
            vec![Subfield::new('a', "abc")?, Subfield::new('a', "hij")?],
        );

        assert!(field.validate().is_ok());

        let field = Field::new(
            Tag::from_str("012A")?,
            None,
            vec![
                Subfield::new('a', "abc")?,
                Subfield::new('b', vec![0, 157])?,
                Subfield::new('a', "hij")?,
            ],
        );

        assert!(field.validate().is_err());

        Ok(())
    }

    #[test]
    fn test_field_write() -> TestResult {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let field = Field::new(
            Tag::from_str("012A")?,
            Some(Occurrence::from_str("/01")?),
            vec![Subfield::new('a', "abc")?, Subfield::new('a', "hij")?],
        );

        field.write(&mut writer)?;

        assert_eq!(
            String::from_utf8(writer.into_inner())?,
            "012A/01 \x1faabc\x1fahij\x1e"
        );

        Ok(())
    }

    #[test]
    fn test_field_to_string() -> TestResult {
        let field = Field::new(
            Tag::from_str("012A")?,
            Some(Occurrence::from_str("/01")?),
            vec![Subfield::new('a', "abc")?, Subfield::new('a', "hij")?],
        );

        assert_eq!(field.to_string(), "012A/01 $aabc$ahij");
        Ok(())
    }
}
