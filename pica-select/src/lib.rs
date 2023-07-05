use std::fmt::{Debug, Display};
use std::iter::repeat;
use std::ops::{Add, Deref, Mul};
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::{char, multispace0, multispace1};
use nom::combinator::{all_consuming, map, map_res, value, verify};
use nom::multi::{fold_many0, separated_list1};
use nom::sequence::{delimited, preceded};
use nom::Finish;
use pica_matcher::subfield_matcher::Matcher;
use pica_matcher::MatcherOptions;
use pica_path::{parse_path, Path};
use pica_record::parser::ParseResult;
use pica_record::Record;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid selector, got `{0}`")]
pub struct ParseSelectorError(String);

#[derive(Debug, PartialEq, Eq)]
pub enum QueryFragment {
    Path(Path),
    Const(String),
}

impl From<Path> for QueryFragment {
    fn from(value: Path) -> Self {
        Self::Path(value)
    }
}

impl From<String> for QueryFragment {
    fn from(value: String) -> Self {
        Self::Const(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Query(Vec<QueryFragment>);

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

impl Deref for Query {
    type Target = Vec<QueryFragment>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

#[derive(Debug, Copy, Clone)]
enum Quotes {
    Single,
    Double,
}

#[derive(Debug, Clone)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWs,
}

/// Parse a non-empty block of text that doesn't include \ or ".
fn parse_literal(
    quotes: Quotes,
) -> impl Fn(&[u8]) -> ParseResult<&str> {
    move |i: &[u8]| {
        let arr = match quotes {
            Quotes::Single => "\'\\",
            Quotes::Double => "\"\\",
        };

        map_res(
            verify(is_not(arr), |s: &[u8]| !s.is_empty()),
            std::str::from_utf8,
        )(i)
    }
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char(
    quotes: Quotes,
) -> impl Fn(&[u8]) -> ParseResult<char> {
    move |i: &[u8]| {
        let val = match quotes {
            Quotes::Single => '"',
            Quotes::Double => '\'',
        };

        preceded(
            char('\\'),
            alt((
                // parse_unicode,
                value('\n', char('n')),
                value('\r', char('r')),
                value('\t', char('t')),
                value('\u{08}', char('b')),
                value('\u{0C}', char('f')),
                value('\\', char('\\')),
                value('/', char('/')),
                value(val, char(val)),
            )),
        )(i)
    }
}

/// Combine parse_literal, parse_escaped_char into a StringFragment.
fn parse_fragment(
    quotes: Quotes,
) -> impl Fn(&[u8]) -> ParseResult<StringFragment> {
    move |i: &[u8]| {
        alt((
            map(parse_literal(quotes), StringFragment::Literal),
            map(
                parse_escaped_char(quotes),
                StringFragment::EscapedChar,
            ),
            value(
                StringFragment::EscapedWs,
                preceded(char('\\'), multispace1),
            ),
        ))(i)
    }
}

fn parse_string_inner(
    quotes: Quotes,
) -> impl Fn(&[u8]) -> ParseResult<String> {
    move |i: &[u8]| {
        fold_many0(
            parse_fragment(quotes),
            String::new,
            |mut string, fragment| {
                match fragment {
                    StringFragment::Literal(s) => string.push_str(s),
                    StringFragment::EscapedChar(c) => string.push(c),
                    StringFragment::EscapedWs => {}
                }
                string
            },
        )(i)
    }
}

fn parse_string_single_quoted(i: &[u8]) -> ParseResult<String> {
    delimited(
        char('\''),
        parse_string_inner(Quotes::Single),
        char('\''),
    )(i)
}

fn parse_string_double_quoted(i: &[u8]) -> ParseResult<String> {
    delimited(char('"'), parse_string_inner(Quotes::Double), char('"'))(
        i,
    )
}

pub(crate) fn parse_string(i: &[u8]) -> ParseResult<String> {
    alt((parse_string_single_quoted, parse_string_double_quoted))(i)
}

fn parse_query_fragment(i: &[u8]) -> ParseResult<QueryFragment> {
    alt((
        map(parse_path, QueryFragment::Path),
        map(parse_string, QueryFragment::Const),
    ))(i)
}

fn parse_query(i: &[u8]) -> ParseResult<Query> {
    map(
        separated_list1(
            delimited(multispace0, char(','), multispace0),
            parse_query_fragment,
        ),
        Query,
    )(i)
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
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            case_ignore: false,
            strsim_threshold: 0.8,
            separator: "|".into(),
            squash: false,
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

    /// Set the squash separator.
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

impl<T: AsRef<[u8]> + Debug + Display> QueryExt for Record<T> {
    /// Performs a query against a PICA+ record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use pica_record::RecordRef;
    /// use pica_select::{Outcome, Query, QueryExt};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let query =
    ///         Query::from_str("003@.0, 012A{(a,b) | a == 'abc'}")?;
    ///     let record = RecordRef::from_bytes(
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
                                .map(|code| {
                                    field
                                        .subfields()
                                        .iter()
                                        .filter(|subfield| {
                                            subfield.code() == *code
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
            .reduce(|acc, e| acc * e)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::assert_finished_and_eq;
    use pica_record::RecordRef;

    use super::*;

    macro_rules! s {
        ($s:expr) => {
            $s.to_string()
        };
    }

    #[test]
    fn test_parse_query() -> anyhow::Result<()> {
        assert_finished_and_eq!(
            parse_query(b"003@.0,012A/*.a"),
            Query(vec![
                Path::new("003@.0").into(),
                Path::new("012A/*.a").into(),
            ])
        );

        assert_finished_and_eq!(
            parse_query(b"003@.0, 012A/*{b, c | a?}"),
            Query(vec![
                Path::new("003@.0").into(),
                Path::new("012A/*{b,c |a?}").into(),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_outcome_from_vec() -> anyhow::Result<()> {
        assert_eq!(
            Outcome::from(vec![s!("abc"), s!("def")]),
            Outcome(vec![vec![s!("abc")], vec![s!("def")]])
        );

        Ok(())
    }

    #[test]
    fn test_outcome_add() -> anyhow::Result<()> {
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

        Ok(())
    }

    #[test]
    fn test_outcome_mul() -> anyhow::Result<()> {
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

        Ok(())
    }

    #[test]
    fn test_query() -> anyhow::Result<()> {
        let options = QueryOptions::default();

        let record =
            RecordRef::new(vec![("012A", None, vec![('a', "1")])]);
        assert_eq!(
            record.query(&Query::from_str("012A.a")?, &options),
            Outcome::from(vec![s!("1")])
        );

        let record = RecordRef::new(vec![(
            "012A",
            None,
            vec![('a', "1"), ('a', "2")],
        )]);
        assert_eq!(
            record.query(&Query::from_str("012A.a")?, &options),
            Outcome::from(vec![s!("1"), s!("2")])
        );

        let record = RecordRef::new(vec![
            ("012A", None, vec![('a', "1")]),
            ("012A", None, vec![('a', "2")]),
        ]);
        assert_eq!(
            record.query(&Query::from_str("012A.a")?, &options),
            Outcome::from(vec![s!("1"), s!("2")])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
        ]);
        assert_eq!(
            record.query(&Query::from_str("003@.0, 012A.a")?, &options),
            Outcome(vec![vec![s!("9"), s!("1")]])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
            ("012A", None, vec![('a', "2")]),
        ]);
        assert_eq!(
            record.query(&Query::from_str("003@.0, 012A.a")?, &options),
            Outcome(vec![
                vec![s!("9"), s!("1")],
                vec![s!("9"), s!("2")],
            ])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('b', "2")]),
        ]);
        assert_eq!(
            record.query(
                &Query::from_str("003@.0, 012A{ (a, b) }")?,
                &options
            ),
            Outcome(vec![vec![s!("9"), s!("1"), s!("2")]])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
        ]);
        assert_eq!(
            record.query(
                &Query::from_str("003@.0, 012A{ (a, b) }")?,
                &options
            ),
            Outcome(vec![vec![s!("9"), s!("1"), s!("")]])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
        ]);
        assert_eq!(
            record.query(
                &Query::from_str("003@.0, 012A{ (a, b) }")?,
                &options
            ),
            Outcome(vec![
                vec![s!("9"), s!("1"), s!("")],
                vec![s!("9"), s!("2"), s!("")],
            ])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
            ("012A", None, vec![('a', "3"), ('b', "4")]),
        ]);
        assert_eq!(
            record.query(
                &Query::from_str("003@.0, 012A{ (a, b) }")?,
                &options
            ),
            Outcome(vec![
                vec![s!("9"), s!("1"), s!("")],
                vec![s!("9"), s!("2"), s!("")],
                vec![s!("9"), s!("3"), s!("4")],
            ])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
            ("012A", None, vec![('a', "3"), ('b', "4"), ('x', "5")]),
        ]);
        assert_eq!(
            record.query(
                &Query::from_str("003@.0, 012A{ (a,b) | x? }")?,
                &options
            ),
            Outcome(vec![vec![s!("9"), s!("3"), s!("4")],])
        );

        let record =
            RecordRef::new(vec![("012A", None, vec![('a', "1")])]);
        assert_eq!(
            record.query(&Query::from_str("012A.a, 'foo'")?, &options),
            Outcome(vec![vec![s!("1"), s!("foo")]])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
            ("012A", None, vec![('a', "3"), ('b', "4"), ('x', "5")]),
        ]);
        assert_eq!(
            record.query(
                &Query::from_str(
                    "003@.0, \"bar\", 012A{ (a,b) | x? }"
                )?,
                &options
            ),
            Outcome(vec![vec![s!("9"), s!("bar"), s!("3"), s!("4")],])
        );

        Ok(())
    }
}
