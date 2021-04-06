use crate::record::{
    parse_field_occurrence, parse_field_tag, parse_subfield_code, ParseResult,
};
use crate::Occurrence;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, map, success};
use nom::sequence::{delimited, preceded, tuple};

use bstr::BString;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum OccurrenceMatcher {
    Occurrence(Occurrence),
    None,
    Any,
}

impl PartialEq<OccurrenceMatcher> for Option<Occurrence> {
    fn eq(&self, other: &OccurrenceMatcher) -> bool {
        match other {
            OccurrenceMatcher::Any => true,
            OccurrenceMatcher::None => self.is_none(),
            OccurrenceMatcher::Occurrence(o1) => {
                if let Some(o2) = self {
                    o1 == o2
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Path {
    pub(crate) tag: BString,
    pub(crate) occurrence: OccurrenceMatcher,
    pub(crate) code: char,
}

#[derive(Debug)]
pub struct ParsePathError(String);

impl std::error::Error for ParsePathError {}

impl fmt::Display for ParsePathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Path {
    pub fn new<T: Into<BString>>(
        tag: T,
        occurrence: OccurrenceMatcher,
        code: char,
    ) -> Path {
        Self {
            tag: tag.into(),
            occurrence,
            code,
        }
    }

    /// Creates a new `Path` from a byte vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Occurrence, OccurrenceMatcher, Path};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let path = Path::from_bytes("003@.0")?;
    ///     assert_eq!(path, Path::new("003@", OccurrenceMatcher::None, '0'));
    ///
    ///     let path = Path::from_bytes("012A/00.0")?;
    ///     assert_eq!(
    ///         path,
    ///         Path::new(
    ///             "012A",
    ///             OccurrenceMatcher::Occurrence(Occurrence::new("00")?),
    ///             '0'
    ///         )
    ///     );
    ///
    ///     let path = Path::from_bytes("012A/*.0")?;
    ///     assert_eq!(path, Path::new("012A", OccurrenceMatcher::Any, '0'));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes<T>(data: T) -> Result<Path, ParsePathError>
    where
        T: Into<Vec<u8>>,
    {
        match parse_path(&data.into()) {
            Err(_) => {
                Err(ParsePathError(String::from("Invalid path expression")))
            }
            Ok((_, path)) => Ok(path),
        }
    }
}

impl FromStr for Path {
    type Err = crate::error::Error;

    /// Parse a `Path` from a string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Path;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let path = Path::from_str("003@.0");
    ///     assert!(path.is_ok());
    ///
    ///     Ok(())
    /// }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_bytes(s)?)
    }
}

fn parse_path(i: &[u8]) -> ParseResult<Path> {
    map(
        all_consuming(delimited(
            multispace0,
            tuple((
                parse_field_tag,
                parse_occurrence_matcher,
                preceded(char('.'), parse_subfield_code),
            )),
            multispace0,
        )),
        |(tag, occurrence, code)| Path::new(tag, occurrence, code),
    )(i)
}

fn parse_occurrence_matcher(i: &[u8]) -> ParseResult<OccurrenceMatcher> {
    alt((
        map(tag(b"/*"), |_| OccurrenceMatcher::Any),
        map(parse_field_occurrence, |x| OccurrenceMatcher::Occurrence(x)),
        success(OccurrenceMatcher::None),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path(b"012A/00.a").unwrap().1,
            Path::new(
                "012A",
                OccurrenceMatcher::Occurrence(Occurrence::new("00").unwrap()),
                'a'
            )
        );
    }

    #[test]
    fn test_parse_occurrence_matcher() {
        assert_eq!(
            parse_occurrence_matcher(b"/00").unwrap().1,
            OccurrenceMatcher::Occurrence(Occurrence::new("00").unwrap())
        );
        assert_eq!(
            parse_occurrence_matcher(b"/*").unwrap().1,
            OccurrenceMatcher::Any
        );
        assert_eq!(
            parse_occurrence_matcher(b"abc").unwrap().1,
            OccurrenceMatcher::None
        );
    }
}
