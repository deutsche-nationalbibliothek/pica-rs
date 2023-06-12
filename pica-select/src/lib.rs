use std::fmt::Debug;
use std::ops::{Add, Deref, Mul};
use std::str::FromStr;

use nom::character::complete::{char, multispace0};
use nom::combinator::{all_consuming, map};
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::Finish;
// use pica_matcher::subfield_matcher::Matcher;
use pica_path::{parse_path, Path};
use pica_record::parser::ParseResult;
// use pica_record::Record;
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

#[derive(Debug, PartialEq, Eq)]
pub struct Outcome<T: AsRef<[u8]>>(Vec<Vec<T>>);

impl<T: AsRef<[u8]>> Deref for Outcome<T> {
    type Target = Vec<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<[u8]> + Clone> Default for Outcome<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T: AsRef<[u8]>> From<Vec<T>> for Outcome<T> {
    fn from(values: Vec<T>) -> Self {
        Self(values.into_iter().map(|v| vec![v]).collect())
    }
}

impl<T: AsRef<[u8]>> Add for Outcome<T> {
    type Output = Outcome<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.0;
        result.extend(rhs.0.into_iter());
        Self(result)
    }
}

impl<T: AsRef<[u8]> + Clone> Mul for Outcome<T> {
    type Output = Outcome<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_empty() {
            return rhs;
        }

        if rhs.is_empty() {
            return self;
        }

        let mut rows: Vec<Vec<T>> = vec![];
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

//#[derive(Debug, PartialEq, Eq)]
// pub struct Outcome<T>(Vec<Vec<T>>);

// impl<T> From<Vec<T>> for Outcome<T> {
//    fn from(value: Vec<T>) -> Self {
//        Self(value.into_iter().map(|value| vec![value]).collect())
//    }
//}

// pub trait QueryExt<T> {
//    fn query(&self, query: &Query) -> Outcome<T>;
//}

// impl<T: AsRef<[u8]> + std::fmt::Debug> QueryExt<T> for Record<T> {
//    /// TODO
//    ///
//    /// # Example
//    ///
//    /// ```rust
//    /// use std::str::FromStr;
//    ///
//    /// use pica_record::RecordRef;
//    /// use pica_select::{Query, QueryExt};
//    ///
//    /// # fn main() { example().unwrap(); }
//    /// fn example() -> anyhow::Result<()> {
//    ///     let query =
//    ///         Query::from_str("003@.0, 012A{ (a,b) | a == 'abc'
// }")?;    ///     let record = RecordRef::from_bytes(
//    ///         b"003@ \x1f01234\x1e012A \x1faabc\x1e\n",
//    ///     )?;
//    ///
//    ///     record.query(&query);
//    ///     Ok(())
//    /// }
//    /// ```
//    fn query(&self, query: &Query) -> Outcome<T> {
//        let options = Default::default();

//        for path in query.iter() {
//            eprintln!("===> path = {:?}", path);

//            let _result = self
//                .iter()
//                .filter(|field| {
//                    path.tag_matcher().is_match(field.tag())
//                        && *path.occurrence_matcher()
//                            == field.occurrence()
//                })
//                .filter(|field| {
//                    if let Some(m) = path.subfield_matcher() {
//                        m.is_match(field.subfields(), &options)
//                    } else {
//                        true
//                    }
//                })
//                .map(|field| {
//                    path.codes()
//                        .iter()
//                        .map(|code| {
//                            field
//                                .subfields()
//                                .iter()
//                                .filter(|subfield| {
//                                    subfield.code() == *code
//                                })
//                                .map(|subfield| subfield.value())
//                                .collect::<Vec<_>>()
//                        })
//                        .map(|values| Outcome::from(values))
//                        .collect::<Vec<_>>()
//                })
//                .collect::<Vec<_>>();
//            eprintln!("result = {:?}", _result);
//        }

//        todo!()
//    }
//}

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
        assert_eq!(
            Outcome::from(vec!["abc", "def"]),
            Outcome(vec![vec!["abc"], vec!["def"]])
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

        let lhs = Outcome(vec![vec!["abc", "def"]]);
        let rhs = Outcome(vec![vec!["123", "456"]]);

        assert_eq!(
            lhs + rhs,
            Outcome(vec![vec!["abc", "def"], vec!["123", "456"]])
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
                vec!["abc", "123"],
                vec!["abc", "456"],
                vec!["def", "123"],
                vec!["def", "456"],
            ])
        );

        let lhs = Outcome(vec![vec!["abc", "def"]]);
        let rhs = Outcome::from(vec!["123", "456"]);

        assert_eq!(
            lhs * rhs,
            Outcome(vec![
                vec!["abc", "def", "123"],
                vec!["abc", "def", "456"],
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
}
