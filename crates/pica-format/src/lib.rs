use std::ops::RangeTo;
use std::str::FromStr;

use pica_matcher::{OccurrenceMatcher, SubfieldMatcher, TagMatcher};
use pica_record::{FieldRef, RecordRef, SubfieldRef};
use thiserror::Error;
use winnow::prelude::*;

mod parse;

pub use parse::parse_format;

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

trait Formatter {
    fn format_subfield(
        &self,
        _subfield: &SubfieldRef,
        _options: &FormatOptions,
    ) -> String;

    fn format_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> String;
}

#[derive(Debug, Clone, PartialEq)]
enum Fragments {
    Group(Group),
    Value(Value),
    List(List),
}

impl Formatter for Fragments {
    fn format_subfield(
        &self,
        subfield: &SubfieldRef,
        options: &FormatOptions,
    ) -> String {
        match self {
            Self::Group(group) => {
                group.format_subfield(subfield, options)
            }
            Self::Value(value) => {
                value.format_subfield(subfield, options)
            }
            Self::List(list) => list.format_subfield(subfield, options),
        }
    }

    fn format_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> String {
        match self {
            Self::Group(group) => group.format_field(field, options),
            Self::Value(value) => value.format_field(field, options),
            Self::List(list) => list.format_field(field, options),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Value {
    codes: Vec<char>,
    prefix: Option<String>,
    suffix: Option<String>,
    bounds: RangeTo<usize>,
}

impl Formatter for Value {
    fn format_subfield(
        &self,
        subfield: &SubfieldRef,
        options: &FormatOptions,
    ) -> String {
        if !self.codes.contains(&subfield.code()) {
            return "".into();
        }

        let mut value = subfield.value().to_string();
        if options.strip_overread_char {
            value = value.replacen('@', "", 1);
        }

        if let Some(ref prefix) = self.prefix {
            value.insert_str(0, prefix);
        }

        if let Some(ref suffix) = self.suffix {
            value.push_str(suffix);
        }

        value
    }

    fn format_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> String {
        let mut acc = String::new();
        let mut cnt = 0;

        for subfield in field.subfields().iter() {
            if !self.codes.contains(&subfield.code()) {
                continue;
            }

            if !self.bounds.contains(&cnt) {
                break;
            }

            let value = self.format_subfield(subfield, options);
            if !value.is_empty() {
                acc.push_str(&value);
                cnt += 1;
            }
        }

        acc
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Group {
    fragments: Box<Fragments>,
    bounds: RangeTo<usize>,
    modifier: Modifier,
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

impl Formatter for Group {
    fn format_subfield(
        &self,
        subfield: &SubfieldRef,
        options: &FormatOptions,
    ) -> String {
        self.fragments.format_subfield(subfield, options)
    }

    fn format_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> String {
        let mut acc = String::new();
        let mut count = 0;

        for subfield in field.subfields().iter() {
            if !self.bounds.contains(&count) {
                break;
            }

            let value = self.format_subfield(subfield, options);
            if !value.is_empty() {
                acc.push_str(&value);
                count += 1;
            }
        }

        if self.modifier.trim {
            acc = acc.trim().to_string();
        }

        if self.modifier.remove_ws {
            acc = acc.replace(' ', "").to_string();
        }

        if self.modifier.lowercase {
            acc = acc.to_lowercase();
        }

        if self.modifier.uppercase {
            acc = acc.to_uppercase();
        }

        acc
    }
}

#[derive(Debug, Clone, PartialEq)]
enum List {
    AndThen(Vec<Fragments>),
    Cons(Vec<Fragments>),
}

impl Formatter for List {
    fn format_subfield(
        &self,
        subfield: &SubfieldRef,
        options: &FormatOptions,
    ) -> String {
        let mut acc = String::new();

        match self {
            Self::AndThen(fragments) => {
                for f in fragments.iter() {
                    let value = f.format_subfield(subfield, options);
                    if value.is_empty() {
                        break;
                    }

                    acc.push_str(&value);
                }
            }
            Self::Cons(fragments) => {
                for f in fragments.iter() {
                    acc.push_str(&f.format_subfield(subfield, options));
                }
            }
        }

        acc
    }
    fn format_field(
        &self,
        field: &FieldRef,
        options: &FormatOptions,
    ) -> String {
        let mut acc = String::new();

        match self {
            Self::AndThen(fragments) => {
                for f in fragments.iter() {
                    let value = f.format_field(field, options);
                    if value.is_empty() {
                        break;
                    }

                    acc.push_str(&value);
                }
            }
            Self::Cons(fragments) => {
                for f in fragments.iter() {
                    acc.push_str(&f.format_field(field, options));
                }
            }
        }

        acc
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

pub trait FormatExt {
    fn format(
        &self,
        format: &Format,
        options: &FormatOptions,
    ) -> Vec<String>;
}

impl FormatExt for RecordRef<'_> {
    fn format(
        &self,
        format: &Format,
        options: &FormatOptions,
    ) -> Vec<String> {
        let mut acc = Vec::new();

        for field in self.iter() {
            if !format.tag_matcher().is_match(field.tag())
                || *format.occurrence_matcher() != field.occurrence()
            {
                continue;
            }

            if let Some(matcher) = format.subfield_matcher() {
                if !matcher
                    .is_match(field.subfields(), &Default::default())
                {
                    continue;
                }
            }

            let value = format.fragments.format_field(field, options);
            if !value.is_empty() {
                acc.push(value);
            }
        }

        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn test_parse_format() -> TestResult {
        let format = Format::new(
            "041A{ x.. <$> (a <*> b)..2 <$> (c <*> d).. | a? }",
        );

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
