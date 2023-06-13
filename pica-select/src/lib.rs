use std::fmt::{Debug, Display};
use std::iter::repeat;
use std::ops::{Add, Deref, Mul};
use std::str::FromStr;

use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, map};
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::Finish;
use pica_matcher::subfield_matcher::Matcher;
use pica_path::{parse_path, Path};
use pica_record::parser::ParseResult;
use pica_record::Record;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid selector, got `{0}`")]
pub struct ParseSelectorError(String);

#[derive(Debug, PartialEq, Eq)]
pub struct Query(Vec<Path>);

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
    type Target = Vec<Path>;

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

fn parse_query(i: &[u8]) -> ParseResult<Query> {
    map(
        separated_list1(
            delimited(multispace0, char(','), multispace0),
            parse_path,
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
        result.extend(rhs.0.into_iter());
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

pub trait QueryExt {
    fn query(&self, query: &Query) -> Outcome;
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
    ///         record.query(&query).into_inner(),
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
    fn query(&self, query: &Query) -> Outcome {
        let options = Default::default();
        let mut outcomes = vec![];

        for path in query.iter() {
            let mut outcome = self
                .iter()
                .filter(|field| {
                    path.tag_matcher().is_match(field.tag())
                        && *path.occurrence_matcher()
                            == field.occurrence()
                })
                .filter(|field| {
                    if let Some(m) = path.subfield_matcher() {
                        m.is_match(field.subfields(), &options)
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
                                .map(|subfield| subfield.value())
                                .collect::<Vec<_>>()
                        })
                        .map(|values| {
                            if !values.is_empty() {
                                Outcome::from(values)
                            } else {
                                Outcome::one()
                            }
                        })
                        .fold(Outcome::default(), |acc, e| acc * e)
                })
                .fold(Outcome::default(), |acc, e| acc + e);

            if outcome.is_empty() {
                outcome = Outcome::ones(path.codes().len());
            }

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
            Query(vec![Path::new("003@.0"), Path::new("012A/*.a")])
        );

        assert_finished_and_eq!(
            parse_query(b"003@.0, 012A/*{b, c | a?}"),
            Query(vec![
                Path::new("003@.0"),
                Path::new("012A/*{b,c |a?}"),
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
        let record =
            RecordRef::new(vec![("012A", None, vec![('a', "1")])]);
        assert_eq!(
            record.query(&Query::from_str("012A.a")?),
            Outcome::from(vec![s!("1")])
        );

        let record = RecordRef::new(vec![(
            "012A",
            None,
            vec![('a', "1"), ('a', "2")],
        )]);
        assert_eq!(
            record.query(&Query::from_str("012A.a")?),
            Outcome::from(vec![s!("1"), s!("2")])
        );

        let record = RecordRef::new(vec![
            ("012A", None, vec![('a', "1")]),
            ("012A", None, vec![('a', "2")]),
        ]);
        assert_eq!(
            record.query(&Query::from_str("012A.a")?),
            Outcome::from(vec![s!("1"), s!("2")])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
        ]);
        assert_eq!(
            record.query(&Query::from_str("003@.0, 012A.a")?),
            Outcome(vec![vec![s!("9"), s!("1")]])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
            ("012A", None, vec![('a', "2")]),
        ]);
        assert_eq!(
            record.query(&Query::from_str("003@.0, 012A.a")?),
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
            record.query(&Query::from_str("003@.0, 012A{ (a, b) }")?),
            Outcome(vec![vec![s!("9"), s!("1"), s!("2")]])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1")]),
        ]);
        assert_eq!(
            record.query(&Query::from_str("003@.0, 012A{ (a, b) }")?),
            Outcome(vec![vec![s!("9"), s!("1"), s!("")]])
        );

        let record = RecordRef::new(vec![
            ("003@", None, vec![('0', "9")]),
            ("012A", None, vec![('a', "1"), ('a', "2")]),
        ]);
        assert_eq!(
            record.query(&Query::from_str("003@.0, 012A{ (a, b) }")?),
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
            record.query(&Query::from_str("003@.0, 012A{ (a, b) }")?),
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
            record
                .query(&Query::from_str("003@.0, 012A{ (a,b) | x? }")?),
            Outcome(vec![vec![s!("9"), s!("3"), s!("4")],])
        );

        Ok(())
    }
}
