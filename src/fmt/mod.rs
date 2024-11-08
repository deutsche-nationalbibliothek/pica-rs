use std::fmt::{self, Display};
use std::ops::RangeTo;

use parser::parse_format;
use smallvec::SmallVec;
use winnow::prelude::*;

use crate::matcher::subfield::SubfieldMatcher;
use crate::matcher::{OccurrenceMatcher, TagMatcher};
use crate::primitives::SubfieldCode;

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

#[derive(Debug, Clone, PartialEq)]
struct Group {
    fragments: Box<Fragments>,
    bounds: RangeTo<usize>,
    modifier: Modifier,
}

#[derive(Debug, Clone, PartialEq)]
struct Value {
    codes: SmallVec<[SubfieldCode; 4]>,
    prefix: Option<String>,
    suffix: Option<String>,
    bounds: RangeTo<usize>,
}

#[derive(Debug, Clone, PartialEq)]
enum List {
    AndThen(Vec<Fragments>),
    Cons(Vec<Fragments>),
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
    type Value: ?Sized;

    /// Returns an iterator over the formatted fields of the record.
    fn format<'a, F, O>(
        &self,
        fmt: F,
        options: &FormatOptions,
    ) -> Result<O, ParseFormatError>
    where
        F: TryInto<Format>,
        O: Iterator<Item = &'a Self::Value>,
        <Self as FormatExt>::Value: 'a;
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    #[cfg(feature = "serde")]
    fn test_format_serde() -> TestResult {
        assert_tokens(
            &Format::new("028@{ a <$> d }")?,
            &[Token::Str("028@{ a <$> d }")],
        );

        Ok(())
    }
}
