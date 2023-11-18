use std::fmt::Debug;
use std::iter::repeat;
use std::ops::{Add, Deref, Mul};

use bstr::ByteSlice;
use pica_matcher::MatcherOptions;
use pica_path::{parse_path, Path};
use pica_record::Record;
use thiserror::Error;
use winnow::ascii::{multispace0, multispace1};
use winnow::combinator::{
    alt, delimited, fold_repeat, preceded, separated,
};
use winnow::error::{ContextError, ParserError};
use winnow::prelude::*;
use winnow::stream::{AsChar, Stream, StreamIsPartial};
use winnow::token::take_till1;

#[derive(Debug, Error)]
#[error("invalid selector, got `{0}`")]
pub struct ParseSelectorError(String);

#[derive(Debug)]
pub enum QueryFragment<'a> {
    Path(Path<'a>),
    Const(String),
}

impl<'a> From<Path<'a>> for QueryFragment<'a> {
    fn from(value: Path<'a>) -> Self {
        Self::Path(value)
    }
}

impl<'a> From<String> for QueryFragment<'a> {
    fn from(value: String) -> Self {
        Self::Const(value)
    }
}

#[derive(Debug)]
pub struct Query<'a>(Vec<QueryFragment<'a>>);

#[derive(Debug, Error)]
#[error("invalid query, got `{0}`")]
pub struct ParseQueryError(String);

impl<'a> Query<'a> {
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
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &'a T) -> Self {
        Self::try_from(data.as_ref()).expect("valid query expression.")
    }
}

impl<'a> Deref for Query<'a> {
    type Target = Vec<QueryFragment<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for Query<'a> {
    type Error = ParseQueryError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        parse_query.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseQueryError(value)
        })
    }
}

impl<'a> From<Path<'a>> for Query<'a> {
    fn from(path: Path<'a>) -> Self {
        Self(vec![path.into()])
    }
}

#[derive(Debug, Copy, Clone)]
enum Quotes {
    Single,
    Double,
}

fn parse_literal<I, E>(
    quotes: Quotes,
) -> impl Parser<I, <I as Stream>::Slice, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar,
    E: ParserError<I>,
{
    match quotes {
        Quotes::Single => take_till1(['\'', '\\']),
        Quotes::Double => take_till1(['"', '\\']),
    }
}

fn parse_escaped_char<I, E>(quotes: Quotes) -> impl Parser<I, char, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar + Clone,
    E: ParserError<I>,
{
    let v = match quotes {
        Quotes::Single => '\'',
        Quotes::Double => '"',
    };

    preceded(
        '\\',
        alt((
            'n'.value('\n'),
            'r'.value('\r'),
            't'.value('\t'),
            'b'.value('\u{08}'),
            'f'.value('\u{0C}'),
            '\\'.value('\\'),
            '/'.value('/'),
            v.value(v),
        )),
    )
}

#[derive(Debug, Clone)]
enum StringFragment<'a> {
    Literal(&'a [u8]),
    EscapedChar(char),
    EscapedWs,
}

fn parse_quoted_fragment<'a, E: ParserError<&'a [u8]>>(
    quotes: Quotes,
) -> impl Parser<&'a [u8], StringFragment<'a>, E> {
    use StringFragment::*;

    alt((
        parse_literal::<&'a [u8], E>(quotes).map(Literal),
        parse_escaped_char::<&'a [u8], E>(quotes).map(EscapedChar),
        preceded('\\', multispace1).value(EscapedWs),
    ))
}

fn parse_quoted_string<'a, E>(
    quotes: Quotes,
) -> impl Parser<&'a [u8], Vec<u8>, E>
where
    E: ParserError<&'a [u8]>,
{
    use StringFragment::*;

    let string_builder = fold_repeat(
        0..,
        parse_quoted_fragment::<E>(quotes),
        Vec::new,
        |mut acc, fragment| {
            match fragment {
                Literal(s) => acc.extend_from_slice(s),
                EscapedChar(c) => acc.push(c as u8),
                EscapedWs => {}
            }
            acc
        },
    );

    match quotes {
        Quotes::Single => delimited('\'', string_builder, '\''),
        Quotes::Double => delimited('"', string_builder, '"'),
    }
}

#[inline]
fn parse_string_single_quoted(i: &mut &[u8]) -> PResult<Vec<u8>> {
    parse_quoted_string::<ContextError>(Quotes::Single).parse_next(i)
}

#[inline]
fn parse_string_double_quoted(i: &mut &[u8]) -> PResult<Vec<u8>> {
    parse_quoted_string::<ContextError>(Quotes::Double).parse_next(i)
}

pub(crate) fn parse_string(i: &mut &[u8]) -> PResult<Vec<u8>> {
    alt((parse_string_single_quoted, parse_string_double_quoted))
        .parse_next(i)
}

fn parse_query_fragment<'a>(
    i: &mut &'a [u8],
) -> PResult<QueryFragment<'a>> {
    alt((
        parse_path.map(QueryFragment::Path),
        parse_string
            .verify_map(|value| String::from_utf8(value).ok())
            .map(QueryFragment::Const),
    ))
    .parse_next(i)
}

fn parse_query<'a>(i: &mut &'a [u8]) -> PResult<Query<'a>> {
    separated(
        1..,
        parse_query_fragment,
        delimited(multispace0, ',', multispace0),
    )
    .map(Query)
    .parse_next(i)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome(Vec<Vec<String>>);

impl Outcome {
    pub fn one() -> Self {
        Self(vec![vec!["".to_string()]])
    }

    pub fn ones(n: usize) -> Self {
        Self(vec![repeat("".to_string()).take(n).collect()])
    }

    pub fn squash(self, sep: &str) -> Self {
        let flattened =
            self.0.into_iter().flatten().collect::<Vec<String>>();

        if flattened.len() > 1
            && !sep.is_empty()
            && flattened.iter().any(|item| item.contains(sep))
        {
            eprintln!(
                "WARNING: A subfield value contains \
                      squash separator '{}'.",
                sep
            );
        }

        Self(vec![vec![flattened.join(sep)]])
    }

    pub fn merge(self, sep: &str) -> Self {
        let result = self.0.clone().into_iter().reduce(|acc, e| {
            let mut result = Vec::new();

            for i in 0..acc.len() {
                let mut value = String::from(&acc[i]);
                value.push_str(sep);
                value.push_str(&e[i]);
                result.push(value)
            }

            result
        });

        Self(vec![result.unwrap()])
    }

    pub fn into_inner(self) -> Vec<Vec<String>> {
        self.0
    }
}

impl Deref for Outcome {
    type Target = Vec<Vec<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ToString> From<Vec<T>> for Outcome {
    fn from(values: Vec<T>) -> Self {
        Self(values.into_iter().map(|v| vec![v.to_string()]).collect())
    }
}

impl Add for Outcome {
    type Output = Outcome;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.0;
        result.extend(rhs.0);
        Self(result)
    }
}

impl Mul for Outcome {
    type Output = Outcome;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_empty() {
            return rhs;
        }

        if rhs.is_empty() {
            return self;
        }

        let mut rows = vec![];
        let xs = self.0;
        let ys = rhs.0;

        for x in xs.into_iter() {
            for y in ys.clone().into_iter() {
                let mut row = x.clone();
                row.extend(y.clone());
                rows.push(row);
            }
        }

        Self(rows)
    }
}

/// Options and flags which can be used to configure a matcher.
#[derive(Debug)]
pub struct QueryOptions {
    pub case_ignore: bool,
    pub strsim_threshold: f64,
    pub separator: String,
    pub squash: bool,
    pub merge: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            case_ignore: false,
            strsim_threshold: 0.8,
            separator: "|".into(),
            squash: false,
            merge: false,
        }
    }
}

impl QueryOptions {
    /// Create new matcher flags.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to ignore case when comparing strings or not.
    pub fn case_ignore(mut self, yes: bool) -> Self {
        self.case_ignore = yes;
        self
    }

    /// Set the similarity threshold for the similar operator (`=*`).
    pub fn strsim_threshold(mut self, threshold: f64) -> Self {
        self.strsim_threshold = threshold;
        self
    }

    /// Whether to squash subfield values or not.
    pub fn squash(mut self, yes: bool) -> Self {
        self.squash = yes;
        self
    }

    /// Whether to merge repeated fields or not.
    pub fn merge(mut self, yes: bool) -> Self {
        self.merge = yes;
        self
    }

    /// Set the squash or merge separator.
    pub fn separator<S: Into<String>>(mut self, sep: S) -> Self {
        self.separator = sep.into();
        self
    }
}

impl From<&QueryOptions> for MatcherOptions {
    fn from(options: &QueryOptions) -> Self {
        Self::new()
            .strsim_threshold(options.strsim_threshold)
            .case_ignore(options.case_ignore)
    }
}

pub trait QueryExt {
    fn query(&self, query: &Query, options: &QueryOptions) -> Outcome;
}

impl QueryExt for Record<'_> {
    /// Performs a query against a PICA+ record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use pica_record::Record;
    /// use pica_select::{Outcome, Query, QueryExt};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let query = Query::new("003@.0, 012A{(a,b) | a == 'abc'}");
    ///     let record = Record::from_bytes(
    ///         b"003@ \x1f01234\x1e012A \x1faabc\x1e\n",
    ///     )?;
    ///
    ///     assert_eq!(
    ///         record.query(&query, &Default::default()).into_inner(),
    ///         vec![vec![
    ///             "1234".to_string(),
    ///             "abc".to_string(),
    ///             "".to_string()
    ///         ]]
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    fn query(&self, query: &Query, options: &QueryOptions) -> Outcome {
        let mut outcomes = vec![];

        for fragment in query.iter() {
            let outcome = match fragment {
                QueryFragment::Const(value) => {
                    Outcome(vec![vec![value.to_owned()]])
                }
                QueryFragment::Path(path) => {
                    let mut outcome = self
                        .iter()
                        .filter(|field| {
                            path.tag_matcher().is_match(field.tag())
                                && *path.occurrence_matcher()
                                    == field.occurrence()
                        })
                        .filter(|field| {
                            if let Some(m) = path.subfield_matcher() {
                                m.is_match(
                                    field.subfields(),
                                    &options.into(),
                                )
                            } else {
                                true
                            }
                        })
                        .map(|field| {
                            path.codes()
                                .iter()
                                .map(|codes| {
                                    field
                                        .subfields()
                                        .iter()
                                        .filter(|subfield| {
                                            codes.contains(
                                                &subfield.code(),
                                            )
                                        })
                                        .map(|subfield| {
                                            subfield.value()
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .map(|values| {
                                    if !values.is_empty() {
                                        Outcome::from(values)
                                    } else {
                                        Outcome::one()
                                    }
                                })
                                .map(|outcome| {
                                    if options.squash {
                                        outcome
                                            .squash(&options.separator)
                                    } else {
                                        outcome
                                    }
                                })
                                .fold(Outcome::default(), |acc, e| {
                                    acc * e
                                })
                        })
                        .fold(Outcome::default(), |acc, e| acc + e);

                    if outcome.is_empty() {
                        outcome = Outcome::ones(path.codes().len());
                    }

                    outcome
                }
            };

            outcomes.push(outcome);
        }

        outcomes
            .into_iter()
            .map(|outcome| {
                if options.merge {
                    outcome.merge(&options.separator)
                } else {
                    outcome
                }
            })
            .reduce(|acc, e| acc * e)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    //     use nom_test_helpers::assert_finished_and_eq;
    //     use pica_record::Record;

    use super::*;

    macro_rules! s {
        ($s:expr) => {
            $s.to_string()
        };
    }

    #[test]
    fn test_outcome_from_vec() {
        assert_eq!(
            Outcome::from(vec![s!("abc"), s!("def")]),
            Outcome(vec![vec![s!("abc")], vec![s!("def")]])
        );
    }

    #[test]
    fn test_outcome_add() {
        let lhs = Outcome::from(vec!["abc", "def"]);
        let rhs = Outcome::from(vec!["123", "456"]);

        assert_eq!(
            lhs + rhs,
            Outcome::from(vec!["abc", "def", "123", "456"])
        );

        let lhs = Outcome(vec![vec![s!("abc"), s!("def")]]);
        let rhs = Outcome(vec![vec![s!("123"), s!("456")]]);

        assert_eq!(
            lhs + rhs,
            Outcome(vec![
                vec![s!("abc"), s!("def")],
                vec![s!("123"), s!("456")]
            ])
        );
    }

    #[test]
    fn test_outcome_mul() {
        let lhs = Outcome::from(vec!["abc", "def"]);
        let rhs = Outcome::from(vec!["123", "456"]);

        assert_eq!(
            lhs * rhs,
            Outcome(vec![
                vec![s!("abc"), s!("123")],
                vec![s!("abc"), s!("456")],
                vec![s!("def"), s!("123")],
                vec![s!("def"), s!("456")],
            ])
        );

        let lhs = Outcome(vec![vec![s!("abc"), s!("def")]]);
        let rhs = Outcome::from(vec!["123", "456"]);

        assert_eq!(
            lhs * rhs,
            Outcome(vec![
                vec![s!("abc"), s!("def"), s!("123")],
                vec![s!("abc"), s!("def"), s!("456")],
            ])
        );

        assert_eq!(
            Outcome::default() * Outcome::from(vec!["123", "456"]),
            Outcome::from(vec!["123", "456"])
        );

        assert_eq!(
            Outcome::from(vec!["123", "456"]) * Outcome::default(),
            Outcome::from(vec!["123", "456"])
        );
    }

    #[test]
    fn test_query() {
        let options = QueryOptions::default();

        let record =
            Record::new(vec![("012A", None, vec![('a', "1")])]);
        assert_eq!(
            record.query(&Query::new("012A.a"), &options),
            Outcome::from(vec![s!("1")])
        );

        let record = Record::new(vec![(
            "012A",
            None,
            vec![('a', "1"), ('a', "2")],
        )]);
        assert_eq!(
            record.query(&Query::new("012A.a"), &options),
            Outcome::from(vec![s!("1"), s!("2")])
        );

        let record = Record::new(vec![
            ("012A", None, vec![('a', "1")]),
            ("012A", None, vec![('a', "2")]),
        ]);
        assert_eq!(
            record.query(&Query::new("012A.a"), &options),
            Outcome::from(vec![s!("1"), s!("2")])
        );

        let record = Record::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
        ]);
        assert_eq!(
            record.query(&Query::new("003@.0, 012A.a"), &options),
            Outcome(vec![vec![s!("9"), s!("1")]])
        );

        let record = Record::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
            ("012A", None, vec![('a', "2")]),
        ]);
        assert_eq!(
            record.query(&Query::new("003@.0, 012A.a"), &options),
            Outcome(vec![
                vec![s!("9"), s!("1")],
                vec![s!("9"), s!("2")],
            ])
        );

        let record = Record::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('b', "2")]),
        ]);
        assert_eq!(
            record
                .query(&Query::new("003@.0, 012A{ (a, b) }"), &options),
            Outcome(vec![vec![s!("9"), s!("1"), s!("2")]])
        );

        let record = Record::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
        ]);
        assert_eq!(
            record
                .query(&Query::new("003@.0, 012A{ (a, b) }"), &options),
            Outcome(vec![vec![s!("9"), s!("1"), s!("")]])
        );

        let record = Record::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
        ]);
        assert_eq!(
            record
                .query(&Query::new("003@.0, 012A{ (a, b) }"), &options),
            Outcome(vec![
                vec![s!("9"), s!("1"), s!("")],
                vec![s!("9"), s!("2"), s!("")],
            ])
        );

        let record = Record::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
            ("012A", None, vec![('a', "3"), ('b', "4")]),
        ]);
        assert_eq!(
            record
                .query(&Query::new("003@.0, 012A{ (a, b) }"), &options),
            Outcome(vec![
                vec![s!("9"), s!("1"), s!("")],
                vec![s!("9"), s!("2"), s!("")],
                vec![s!("9"), s!("3"), s!("4")],
            ])
        );

        let record = Record::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
            ("012A", None, vec![('a', "3"), ('b', "4"), ('x', "5")]),
        ]);
        assert_eq!(
            record.query(
                &Query::new("003@.0, 012A{ (a,b) | x? }"),
                &options
            ),
            Outcome(vec![vec![s!("9"), s!("3"), s!("4")],])
        );

        let record =
            Record::new(vec![("012A", None, vec![('a', "1")])]);
        assert_eq!(
            record.query(&Query::new("012A.a, 'foo'"), &options),
            Outcome(vec![vec![s!("1"), s!("foo")]])
        );

        let record = Record::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
            ("012A", None, vec![('a', "3"), ('b', "4"), ('x', "5")]),
        ]);
        assert_eq!(
            record.query(
                &Query::new("003@.0, \"bar\", 012A{ (a,b) | x? }"),
                &options
            ),
            Outcome(vec![vec![s!("9"), s!("bar"), s!("3"), s!("4")],])
        );
    }
}
