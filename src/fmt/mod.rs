use std::fmt::{self, Display};
use std::ops::RangeTo;

use bstr::{BString, ByteSlice};
use parser::parse_format;
use smallvec::SmallVec;
use winnow::prelude::*;

use crate::matcher::subfield::SubfieldMatcher;
use crate::matcher::{OccurrenceMatcher, TagMatcher};
use crate::primitives::{FieldRef, RecordRef, SubfieldCode};

mod parser;

/// An error that can occur when parsing a format expression.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("{0}")]
pub struct ParseFormatError(pub(crate) String);

#[derive(Debug, Clone, PartialEq)]
pub struct Format {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    raw_format: String,
    fragments: Fragments,
}

impl Format {
    /// Creates a new [Format].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// format expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let _fmt = Format::new("028[A@]{ a }")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(fmt: &str) -> Result<Self, ParseFormatError> {
        parse_format.parse(fmt.as_bytes()).map_err(|_| {
            ParseFormatError(format!("invalid format '{fmt}'"))
        })
    }

    pub fn fmt_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> Option<BString> {
        let value = self.fragments.fmt_field(field, options);
        if !value.is_empty() {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatOptions {
    strip_overread_char: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            strip_overread_char: true,
        }
    }
}

impl FormatOptions {
    /// Creates a new [FormatOptions] with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to strip the overread character '@' from a value or not.
    pub fn strip_overread_char(mut self, yes: bool) -> Self {
        self.strip_overread_char = yes;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Fragments {
    Group(Group),
    Value(Value),
    List(List),
}

impl Fragments {
    fn fmt_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> BString {
        match self {
            Self::Value(v) => v.fmt_field(field, options),
            Self::List(l) => l.fmt_field(field, options),
            Self::Group(g) => g.fmt_field(field, options),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Group {
    fragments: Box<Fragments>,
    bounds: RangeTo<usize>,
    modifier: Modifier,
}

impl Group {
    fn fmt_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> BString {
        let mut value = match *self.fragments {
            Fragments::Value(ref v) => v.fmt_field(field, options),
            Fragments::List(ref l) => l.fmt_field(field, options),
            Fragments::Group(_) => unreachable!(),
        };

        self.modifier.modify(&mut value);
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Value {
    codes: SmallVec<[SubfieldCode; 4]>,
    prefix: Option<String>,
    suffix: Option<String>,
    bounds: RangeTo<usize>,
    modifier: Modifier,
}

impl Value {
    fn fmt_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> BString {
        let mut value = BString::new(vec![]);
        let mut cnt = 0;

        for subfield in field.subfields() {
            if !self.bounds.contains(&cnt) {
                break;
            }

            if !self.codes.contains(subfield.code()) {
                continue;
            }

            if let Some(ref prefix) = self.prefix {
                value.extend_from_slice(prefix.as_bytes());
            }

            if options.strip_overread_char {
                value.extend_from_slice(
                    &subfield.value().replacen("@", "", 1),
                )
            } else {
                value.extend_from_slice(subfield.value())
            }

            if let Some(ref suffix) = self.suffix {
                value.extend_from_slice(suffix.as_bytes());
            }

            cnt += 1;
        }

        self.modifier.modify(&mut value);
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
enum List {
    AndThen(Vec<Fragments>),
    Cons(Vec<Fragments>),
}

impl List {
    fn fmt_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> BString {
        let mut acc = BString::new(vec![]);

        match self {
            Self::AndThen(f) => {
                for fragments in f.iter() {
                    let value = fragments.fmt_field(field, options);
                    if value.is_empty() {
                        break;
                    }

                    acc.extend_from_slice(&value);
                }
            }
            Self::Cons(f) => {
                for fragments in f.iter() {
                    acc.extend_from_slice(
                        &fragments.fmt_field(field, options),
                    );
                }
            }
        }

        acc
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Modifier {
    lowercase: bool,
    uppercase: bool,
    remove_ws: bool,
    trim: bool,
}

impl Modifier {
    pub(crate) fn lowercase(&mut self, yes: bool) -> &mut Self {
        self.lowercase = yes;
        self
    }

    pub(crate) fn uppercase(&mut self, yes: bool) -> &mut Self {
        self.uppercase = yes;
        self
    }

    pub(crate) fn remove_ws(&mut self, yes: bool) -> &mut Self {
        self.remove_ws = yes;
        self
    }

    pub(crate) fn trim(&mut self, yes: bool) -> &mut Self {
        self.trim = yes;
        self
    }

    pub(crate) fn modify(&self, value: &mut BString) {
        if self.trim {
            *value = BString::from(value.trim());
        }

        if self.remove_ws {
            *value = BString::from(value.replace(" ", ""));
        }

        if self.lowercase {
            *value = BString::from(value.to_lowercase());
        }

        if self.uppercase {
            *value = BString::from(value.to_uppercase());
        }
    }
}

impl Display for Format {
    /// Formats the [Format] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let fmt = Format::new("028@{ a <$> d }")?;
    /// assert_eq!(fmt.to_string(), "028@{ a <$> d }");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_format)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Format {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Format {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

pub trait FormatExt {
    type Value;

    /// Returns an iterator over the formatted fields of the record.
    fn format(
        &self,
        format: &Format,
        options: &FormatOptions,
    ) -> impl Iterator<Item = Self::Value>;
}

impl FormatExt for RecordRef<'_> {
    type Value = BString;

    fn format(
        &self,
        format: &Format,
        options: &FormatOptions,
    ) -> impl Iterator<Item = Self::Value> {
        self.fields()
            .iter()
            .filter(|field| {
                let retval = format.tag_matcher.is_match(field.tag())
                    && format
                        .occurrence_matcher
                        .is_match(field.occurrence());

                if let Some(ref matcher) = format.subfield_matcher {
                    // FIXME:
                    let options = Default::default();

                    retval
                        && matcher.is_match(field.subfields(), &options)
                } else {
                    retval
                }
            })
            .filter_map(|field| format.fmt_field(field, options))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::OnceLock;
    use std::{env, fs};

    use serde_test::{assert_tokens, Token};

    use super::*;

    type TestResult = anyhow::Result<()>;

    fn ada_lovelace() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let path =
                Path::new(&manifest_dir).join("tests/data/ada.dat");
            fs::read_to_string(&path).unwrap().as_bytes().to_vec()
        })
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_format_serde() -> TestResult {
        assert_tokens(
            &Format::new("028@{ a <$> d }")?,
            &[Token::Str("028@{ a <$> d }")],
        );

        Ok(())
    }

    #[test]
    fn test_format_value() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let options = FormatOptions::default();

        let format = Format::new("028A{ a }")?;
        assert_eq!(
            record.format(&format, &options).collect::<Vec<_>>(),
            vec!["Lovelace"]
        );

        let format = Format::new("028A{ ?t *..2 ' ' }")?;
        assert_eq!(
            record.format(&format, &options).collect::<Vec<_>>(),
            vec!["Ada King of"]
        );

        Ok(())
    }

    #[test]
    fn test_format_list() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let options = FormatOptions::default();

        let format = Format::new("028A{ d ' ' <$> a }")?;
        assert_eq!(
            record.format(&format, &options).collect::<Vec<_>>(),
            vec!["Ada King Lovelace"]
        );

        let format = Format::new("028A{ x  <$> ' ' a }")?;
        let values: Vec<_> = record.format(&format, &options).collect();
        assert!(values.is_empty());

        let format = Format::new("028A{ d ' ' <*> a }")?;
        assert_eq!(
            record.format(&format, &options).collect::<Vec<_>>(),
            vec!["Ada King Lovelace"]
        );

        let format = Format::new("028A{ x <*> ' 'a }")?;
        assert_eq!(
            record.format(&format, &options).collect::<Vec<_>>(),
            vec![" Lovelace"]
        );

        let format = Format::new("028A{ d ' ' <$> c ' ' <*> a }")?;
        assert_eq!(
            record.format(&format, &options).collect::<Vec<_>>(),
            vec!["Ada King of Lovelace"]
        );
        Ok(())
    }

    #[test]
    fn test_format_group() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let options = FormatOptions::default();

        let format =
            Format::new("028A{ a <$> ( ', ' d <*> ' (' c ')' ) }")?;
        assert_eq!(
            record.format(&format, &options).collect::<Vec<_>>(),
            vec!["Lovelace, Ada King (of)"]
        );

        let format =
            Format::new("028A{ (a <$> ( ', ' d <*> ' (' c ')' )) }")?;
        assert_eq!(
            record.format(&format, &options).collect::<Vec<_>>(),
            vec!["Lovelace, Ada King (of)"]
        );

        Ok(())
    }
}
