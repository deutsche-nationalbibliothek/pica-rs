use std::ops::Deref;
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
        |s| Query(s),
    )(i)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Outcome<T>(Vec<Vec<T>>);

impl<T> From<Vec<T>> for Outcome<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value.into_iter().map(|value| vec![value]).collect())
    }
}

pub trait QueryExt<T> {
    fn query(&self, query: &Query) -> Outcome<T>;
}

impl<T: AsRef<[u8]> + std::fmt::Debug> QueryExt<T> for Record<T> {
    /// TODO
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use pica_record::RecordRef;
    /// use pica_select::{Query, QueryExt};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let query =
    ///         Query::from_str("003@.0, 012A{ (a,b) | a == 'abc' }")?;
    ///     let record = RecordRef::from_bytes(
    ///         b"003@ \x1f01234\x1e012A \x1faabc\x1e\n",
    ///     )?;
    ///
    ///     record.query(&query);
    ///     Ok(())
    /// }
    /// ```
    fn query(&self, query: &Query) -> Outcome<T> {
        let options = Default::default();

        for path in query.iter() {
            eprintln!("===> path = {:?}", path);

            let _result = self
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
                        .map(|values| Outcome::from(values))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            eprintln!("result = {:?}", _result);
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::assert_finished_and_eq;

    use super::*;

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
        eprintln!(
            "data = {:?}",
            Outcome::from(vec![vec!["abc", "def"]])
        );

        assert!(false);
        Ok(())
    }
}
