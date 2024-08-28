use std::str::FromStr;

use parse::parse_format;
use pica_matcher::{OccurrenceMatcher, SubfieldMatcher, TagMatcher};
use pica_record::{ByteRecord, FieldRef};
use thiserror::Error;
use winnow::prelude::*;

mod parse;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0} is not a valid format string")]
pub struct ParseFormatError(String);

/// A pica format expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Format {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    fragments: Fragments,
}

impl Format {
    /// Create a new format from the given format string.
    ///
    /// # Panics
    ///
    /// If the give format string is invalid this function panics. To
    /// catch the parse error use `Format::from_str`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_format::Format;
    ///
    /// # fn main() {
    /// let format = Format::new("041[A@]{ a <$> b | a? }");
    /// # }
    /// ```
    pub fn new(fmt: &str) -> Self {
        Self::from_str(fmt).expect("valid format expression")
    }

    /// Returns the tag matcher of the format expression.
    pub fn tag_matcher(&self) -> &TagMatcher {
        &self.tag_matcher
    }

    /// Returns the occurrence matcher of the format expression.
    pub fn occurrence_matcher(&self) -> &OccurrenceMatcher {
        &self.occurrence_matcher
    }

    /// Retruns the subfield matcher of the format expression.
    pub fn subfield_matcher(&self) -> Option<&SubfieldMatcher> {
        self.subfield_matcher.as_ref()
    }
}

impl FromStr for Format {
    type Err = ParseFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_format
            .parse(s.as_bytes())
            .map_err(|_| ParseFormatError(s.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Fragments {
    Group(Group),
    Value(Value),
    List(List),
}

#[derive(Debug, Clone, PartialEq)]
struct Value {
    codes: Vec<char>,
    prefix: Option<String>,
    suffix: Option<String>,
}

impl Value {
    fn format(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> Option<String> {
        let Some(subfield) = self.codes.iter().find_map(|code| {
            field.find(|subfield| subfield.code() == *code)
        }) else {
            return None;
        };

        let mut value = subfield.value().to_string();
        if value.is_empty() {
            return None;
        }

        if options.strip_overread_char {
            value = value.replacen('@', "", 1);
        }

        if let Some(ref prefix) = self.prefix {
            value.insert_str(0, prefix);
        }

        if let Some(ref suffix) = self.suffix {
            value.push_str(suffix)
        }

        Some(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Group {
    fragments: Box<Fragments>,
}

#[derive(Debug, Clone, PartialEq)]
enum List {
    AndThen(Vec<Fragments>),
    Cons(Vec<Fragments>),
}

impl List {
    fn format(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> Option<String> {
        let mut acc = String::new();

        match self {
            Self::AndThen(fragments) => {
                for f in fragments.iter() {
                    let Some(value) = f.format(field, options) else {
                        break;
                    };

                    acc.push_str(&value);
                }
            }
            Self::Cons(fragments) => {
                for f in fragments.iter() {
                    if let Some(value) = f.format(field, options) {
                        acc.push_str(&value);
                    };
                }
            }
        }

        if !acc.is_empty() {
            Some(acc)
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to strip the overread character '@' from a value or not.
    pub fn strip_overread_char(mut self, yes: bool) -> Self {
        self.strip_overread_char = yes;
        self
    }
}

impl Fragments {
    fn format(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> Option<String> {
        match self {
            Self::Value(value) => value.format(field, options),
            Self::List(list) => list.format(field, options),
            Self::Group(Group { fragments }) => {
                fragments.format(field, options)
            }
        }
    }
}

pub trait FormatExt {
    fn format(
        &self,
        format: &Format,
        options: &FormatOptions,
    ) -> Vec<String>;
}

impl FormatExt for ByteRecord<'_> {
    fn format(
        &self,
        format: &Format,
        options: &FormatOptions,
    ) -> Vec<String> {
        self.iter()
            .filter(|field| field.tag() == format.tag_matcher())
            .filter(|field| {
                *format.occurrence_matcher() == field.occurrence()
            })
            .filter(|field| {
                if let Some(m) = format.subfield_matcher() {
                    m.is_match(field.subfields(), &Default::default())
                } else {
                    true
                }
            })
            .filter_map(|field| format.fragments.format(field, options))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn test_parse_format() -> TestResult {
        let format =
            Format::new("041A{ (a <*> b) <$> (c <*> d) | a? }");

        assert_eq!(format.tag_matcher(), &TagMatcher::new("041A"));
        assert_eq!(
            format.occurrence_matcher(),
            &OccurrenceMatcher::None
        );
        assert_eq!(
            format.subfield_matcher().unwrap(),
            &SubfieldMatcher::new("a?")
        );

        Ok(())
    }
}
