use std::str::FromStr;

use nom::branch::alt;
use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, map, opt};
use nom::error::ParseError;
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{Finish, IResult};
use pica_matcher::parser::{
    parse_occurrence_matcher, parse_tag_matcher,
};
use pica_matcher::subfield_matcher::{parse_subfield_matcher, Matcher};
use pica_matcher::{
    MatcherOptions, OccurrenceMatcher, SubfieldMatcher, TagMatcher,
};
use pica_record::parser::{parse_subfield_code, ParseResult};
use pica_record::Record;
#[cfg(feature = "serde")]
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid path expression, got `{0}`")]
pub struct ParsePathError(String);

#[derive(Debug, PartialEq, Eq)]
pub struct Path {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
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

/// Strip whitespaces from the beginning and end.
fn ws<'a, F: 'a, O, E: ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(multispace0, inner, multispace0)
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

fn parse_path_simple(i: &[u8]) -> ParseResult<Path> {
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
            subfield_matcher: None,
            codes: c,
        },
    )(i)
}

fn parse_path_matcher_old(i: &[u8]) -> ParseResult<Path> {
    map(
        delimited(
            multispace0,
            tuple((
                parse_tag_matcher,
                parse_occurrence_matcher,
                delimited(
                    ws(char('{')),
                    pair(
                        opt(terminated(
                            parse_subfield_matcher,
                            ws(char(',')),
                        )),
                        separated_list1(
                            ws(char(',')),
                            parse_subfield_code,
                        ),
                    ),
                    ws(char('}')),
                ),
            )),
            multispace0,
        ),
        |(t, o, (m, c))| Path {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: m,
            codes: c,
        },
    )(i)
}

fn parse_path_matcher_new(i: &[u8]) -> ParseResult<Path> {
    map(
        delimited(
            multispace0,
            tuple((
                parse_tag_matcher,
                parse_occurrence_matcher,
                delimited(
                    ws(char('{')),
                    pair(
                        separated_list1(
                            ws(char(',')),
                            parse_subfield_code,
                        ),
                        opt(preceded(
                            ws(char('|')),
                            parse_subfield_matcher,
                        )),
                    ),
                    ws(char('}')),
                ),
            )),
            multispace0,
        ),
        |(t, o, (c, m))| Path {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: m,
            codes: c,
        },
    )(i)
}

fn parse_path(i: &[u8]) -> ParseResult<Path> {
    alt((
        parse_path_matcher_new,
        parse_path_matcher_old,
        parse_path_simple,
    ))(i)
}

pub trait PathExt<T: AsRef<[u8]>> {
    fn path(&self, path: &Path, options: &MatcherOptions) -> Vec<&T>;

    /// Returns the idn of the record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bstr::ByteSlice;
    /// use pica_path::{Path, PathExt};
    /// use pica_record::ByteRecord;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let record =
    ///         ByteRecord::from_bytes(b"003@ \x1f0123456789X\x1e\n")?;
    ///     assert_eq!(record.idn(), Some(&b"123456789X".as_bstr()));
    ///
    ///     let record = ByteRecord::from_bytes(b"002@ \x1f0Olfo\x1e\n")?;
    ///     assert_eq!(record.idn(), None);
    ///     Ok(())
    /// }
    /// ```
    fn idn(&self) -> Option<&T> {
        self.path(&Path::new("003@.0"), &Default::default())
            .first()
            .copied()
    }
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
    ///         record.path(&Path::new("012A/*.a"), &Default::default()),
    ///         vec![
    ///             &BString::from("123"),
    ///             &BString::from("456"),
    ///             &BString::from("789")
    ///         ]
    ///     );
    ///     Ok(())
    /// }
    /// ```
    fn path(&self, path: &Path, options: &MatcherOptions) -> Vec<&T> {
        self.iter()
            .filter(|field| {
                path.tag_matcher == field.tag()
                    && path.occurrence_matcher == field.occurrence()
            })
            .filter(|field| {
                if let Some(ref matcher) = path.subfield_matcher {
                    matcher.is_match(field.subfields(), options)
                } else {
                    true
                }
            })
            .flat_map(|field| field.subfields())
            .filter_map(|subfield| {
                if path.codes.contains(&subfield.code()) {
                    Some(subfield.value())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Path::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::assert_finished_and_eq;

    use super::*;

    #[test]
    fn test_parse_path() -> anyhow::Result<()> {
        assert_finished_and_eq!(
            parse_path(b"012A/*{a?, b}"),
            Path {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: Some(SubfieldMatcher::new("a?")?),
                codes: vec!['b']
            }
        );

        assert_finished_and_eq!(
            parse_path(b"012A/*{b | a?}"),
            Path {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: Some(SubfieldMatcher::new("a?")?),
                codes: vec!['b']
            }
        );

        assert_finished_and_eq!(
            parse_path(b"012A/*{a?, b, c}"),
            Path {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: Some(SubfieldMatcher::new("a?")?),
                codes: vec!['b', 'c']
            }
        );

        assert_finished_and_eq!(
            parse_path(b"012A/*{b, c | a?}"),
            Path {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: Some(SubfieldMatcher::new("a?")?),
                codes: vec!['b', 'c']
            }
        );

        assert_finished_and_eq!(
            parse_path(b"012A/*.a"),
            Path {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: None,
                codes: vec!['a']
            }
        );

        assert_finished_and_eq!(
            parse_path(b"012A/01.a"),
            Path {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/01")?,
                subfield_matcher: None,
                codes: vec!['a']
            }
        );
        assert_finished_and_eq!(
            parse_path(b"012A.a"),
            Path {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: None,
                codes: vec!['a']
            }
        );

        Ok(())
    }
}
