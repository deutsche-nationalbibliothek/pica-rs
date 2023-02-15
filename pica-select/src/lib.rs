use std::str::FromStr;

use nom::branch::alt;
use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, map, opt};
use nom::error::ParseError;
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::{Finish, IResult};
use pica_matcher::parser::{
    parse_occurrence_matcher, parse_tag_matcher,
};
use pica_matcher::subfield_matcher::parse_subfield_matcher;
use pica_matcher::{
    MatcherOptions, OccurrenceMatcher, SubfieldMatcher, TagMatcher,
};
use pica_record::parser::{parse_subfield_code, ParseResult};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid selector, got `{0}`")]
pub struct ParseSelectorError(String);

#[derive(Debug, PartialEq, Eq)]
pub struct Selector {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    codes: Vec<char>,
}

impl Selector {
    /// Create a new selector from a string slice.
    ///
    /// # Panics
    ///
    /// This methods panics on invalid selector expressions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_select::Selector;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let _selector = Selector::new("003@.0");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Self {
        Self::from_str(data).expect("valid selector expression.")
    }
}

impl FromStr for Selector {
    type Err = ParseSelectorError;

    /// Create a new selector from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_select::Selector;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let _selector = "012A/*.[abc]"
    ///         .parse::<Selector>()
    ///         .expect("valid path expression");
    ///     Ok(())
    /// }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        all_consuming(parse_selector)(s.as_bytes())
            .finish()
            .map_err(|_| ParseSelectorError(s.into()))
            .map(|(_, selector)| selector)
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

fn parse_selector_simple(i: &[u8]) -> ParseResult<Selector> {
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
        |(t, o, c)| Selector {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: None,
            codes: c,
        },
    )(i)
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

fn parse_selector_predicate_inner(
    i: &[u8],
) -> ParseResult<(Vec<char>, Option<SubfieldMatcher>)> {
    delimited(
        ws(char('{')),
        pair(
            alt((
                separated_list1(ws(char(',')), parse_subfield_code),
                delimited(
                    ws(char('(')),
                    separated_list1(ws(char(',')), parse_subfield_code),
                    ws(char(')')),
                ),
            )),
            opt(preceded(ws(char('|')), parse_subfield_matcher)),
        ),
        ws(char('}')),
    )(i)
}
fn parse_selector_predicate(i: &[u8]) -> ParseResult<Selector> {
    map(
        delimited(
            multispace0,
            tuple((
                parse_tag_matcher,
                parse_occurrence_matcher,
                parse_selector_predicate_inner,
            )),
            multispace0,
        ),
        |(t, o, (c, m))| Selector {
            tag_matcher: t,
            occurrence_matcher: o,
            subfield_matcher: m,
            codes: c,
        },
    )(i)
}

fn parse_selector(i: &[u8]) -> ParseResult<Selector> {
    alt((parse_selector_simple, parse_selector_predicate))(i)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Query(Vec<Selector>);

#[derive(Debug, Error)]
#[error("invalid query, got `{0}`")]
pub struct ParseQueryError(String);

impl Query {
    /// Create a new select query from a string slice.
    ///
    /// # Panics
    ///
    /// This methods panics on invalid query expressions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_select::Query;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let _query =
    ///         Query::new("003@.0, 012A{ (a,b) | a? && b == 'foo' }");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Self {
        Self::from_str(data).expect("valid query expression.")
    }
}

impl FromStr for Query {
    type Err = ParseQueryError;

    /// Create a new query from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_select::Query;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let _query = "003@.0, 012A/*.[abc]"
    ///         .parse::<Query>()
    ///         .expect("valid query expression");
    ///     Ok(())
    /// }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        all_consuming(parse_query)(s.as_bytes())
            .finish()
            .map_err(|_| ParseQueryError(s.into()))
            .map(|(_, selector)| selector)
    }
}

fn parse_query(i: &[u8]) -> ParseResult<Query> {
    map(separated_list1(ws(char(',')), parse_selector), |s| Query(s))(i)
}

pub trait SelectExt<T: AsRef<[u8]>> {
    fn select(
        &self,
        query: &Query,
        options: &MatcherOptions,
    ) -> Vec<&T>;
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::assert_finished_and_eq;

    use super::*;

    #[test]
    fn test_parse_selector() -> anyhow::Result<()> {
        assert_finished_and_eq!(
            parse_selector(b"012A/*.a"),
            Selector {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: None,
                codes: vec!['a']
            }
        );

        assert_finished_and_eq!(
            parse_selector(b"012A/*{b, c | a?}"),
            Selector {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: Some(SubfieldMatcher::new("a?")?),
                codes: vec!['b', 'c']
            }
        );

        assert_finished_and_eq!(
            parse_selector(b"012A/*{ b, c }"),
            Selector {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: None,
                codes: vec!['b', 'c']
            }
        );

        assert_finished_and_eq!(
            parse_selector(b"012A/*{ (b, c) | a?}"),
            Selector {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: Some(SubfieldMatcher::new("a?")?),
                codes: vec!['b', 'c']
            }
        );

        assert_finished_and_eq!(
            parse_selector(b"012A/*{ (b, c) }"),
            Selector {
                tag_matcher: TagMatcher::new("012A")?,
                occurrence_matcher: OccurrenceMatcher::new("/*")?,
                subfield_matcher: None,
                codes: vec!['b', 'c']
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_query() -> anyhow::Result<()> {
        assert_finished_and_eq!(
            parse_query(b"003@.0, 012A/*.a"),
            Query(vec![
                Selector {
                    tag_matcher: TagMatcher::new("003@")?,
                    occurrence_matcher: OccurrenceMatcher::None,
                    subfield_matcher: None,
                    codes: vec!['0']
                },
                Selector {
                    tag_matcher: TagMatcher::new("012A")?,
                    occurrence_matcher: OccurrenceMatcher::new("/*")?,
                    subfield_matcher: None,
                    codes: vec!['a']
                },
            ])
        );

        assert_finished_and_eq!(
            parse_query(b"003@.0, 012A/*{b, c | a?}"),
            Query(vec![
                Selector {
                    tag_matcher: TagMatcher::new("003@")?,
                    occurrence_matcher: OccurrenceMatcher::None,
                    subfield_matcher: None,
                    codes: vec!['0']
                },
                Selector {
                    tag_matcher: TagMatcher::new("012A")?,
                    occurrence_matcher: OccurrenceMatcher::new("/*")?,
                    subfield_matcher: Some(SubfieldMatcher::new("a?")?),
                    codes: vec!['b', 'c']
                }
            ])
        );

        Ok(())
    }
}
