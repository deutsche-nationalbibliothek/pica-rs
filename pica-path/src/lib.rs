use std::str::FromStr;

use nom::branch::alt;
use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, map};
use nom::multi::many1;
use nom::sequence::{delimited, preceded, tuple};
use nom::Finish;
use pica_matcher::parser::{
    parse_occurrence_matcher, parse_tag_matcher,
};
use pica_matcher::{OccurrenceMatcher, TagMatcher};
use pica_record::parser::{parse_subfield_code, ParseResult};
use pica_record::Record;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid path expression, got `{0}`")]
pub struct ParsePathError(String);

#[derive(Debug, PartialEq, Eq)]
pub struct Path {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    codes: Vec<char>,
}

impl Path {
    /// Create a new path from a string slice.
    ///
    /// # Panics
    ///
    /// This methods panics on invalid path expressions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_path::Path;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let _path = Path::new("003@.0");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Self {
        Self::from_str(data).expect("valid path expression.")
    }
}

impl FromStr for Path {
    type Err = ParsePathError;

    /// Create a new path from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_path::Path;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let _path = "012A/01-03.[abc]"
    ///         .parse::<Path>()
    ///         .expect("valid path expression");
    ///     Ok(())
    /// }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        all_consuming(parse_path)(s.as_bytes())
            .finish()
            .map_err(|_| ParsePathError(s.into()))
            .map(|(_, matcher)| matcher)
    }
}

fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    preceded(
        char('.'),
        alt((
            map(parse_subfield_code, |code| vec![code]),
            delimited(char('['), many1(parse_subfield_code), char(']')),
        )),
    )(i)
}

fn parse_path(i: &[u8]) -> ParseResult<Path> {
    map(
        delimited(
            multispace0,
            tuple((
                parse_tag_matcher,
                parse_occurrence_matcher,
                parse_subfield_codes,
            )),
            multispace0,
        ),
        |(t, o, c)| Path {
            tag_matcher: t,
            occurrence_matcher: o,
            codes: c,
        },
    )(i)
}

pub trait PathExt<T: AsRef<[u8]>> {
    fn path(&self, path: &Path) -> Vec<&T>;
}

impl<T: AsRef<[u8]>> PathExt<T> for Record<T> {
    /// Returns all subfield values which satisfies the path matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bstr::BString;
    /// use pica_path::{Path, PathExt};
    /// use pica_record::RecordRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record = RecordRef::new(vec![
    ///         ("012A", None, vec![('a', "123"), ('a', "456")]),
    ///         ("012A", Some("01"), vec![('a', "789"), ('b', "xyz")]),
    ///     ]);
    ///
    ///     assert_eq!(
    ///         record.path(&Path::new("012A/*.a")),
    ///         vec![
    ///             &BString::from("123"),
    ///             &BString::from("456"),
    ///             &BString::from("789")
    ///         ]
    ///     );
    ///     Ok(())
    /// }
    /// ```
    fn path(&self, path: &Path) -> Vec<&T> {
        self.iter()
            .filter(|field| {
                path.tag_matcher == field.tag()
                    && path.occurrence_matcher == field.occurrence()
            })
            .map(|field| field.subfields())
            .flatten()
            .filter(|subfield| path.codes.contains(&subfield.code()))
            .map(|subfield| subfield.value())
            .collect::<Vec<&T>>()
    }
}
