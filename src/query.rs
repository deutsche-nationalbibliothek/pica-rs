use std::fmt::{self, Display};
use std::ops::{Add, Deref, Mul};

use bstr::{BString, ByteSlice, ByteVec};
use winnow::combinator::{alt, separated};
use winnow::{ModalResult, Parser};

use crate::StringRecord;
#[cfg(feature = "unstable")]
use crate::fmt::{Format, FormatExt, FormatOptions, parse_format};
use crate::matcher::MatcherOptions;
use crate::parser::{parse_string, ws};
use crate::path::{Path, parse_path};
use crate::primitives::RecordRef;

/// An error that can occur when parsing a query expression.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("{0}")]
pub struct ParseQueryError(pub(crate) String);

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    fragments: Vec<Fragment>,
    raw_query: String,
}

impl Query {
    /// Creates a new [Query].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// query expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let _query = Query::new("003@.0,002@.0")?;
    /// let _query = Query::new("'x', 002@.0")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new<S: AsRef<str>>(
        query: S,
    ) -> Result<Self, ParseQueryError> {
        let query = query.as_ref();
        parse_query.parse(query.as_bytes()).map_err(|_| {
            ParseQueryError(format!("invalid query '{query}'"))
        })
    }
}

fn parse_query(i: &mut &[u8]) -> ModalResult<Query> {
    separated(1.., parse_fragment, ws(','))
        .with_taken()
        .map(|(fragments, raw_query)| {
            let raw_query = raw_query.to_str().unwrap().to_string();
            Query {
                fragments,
                raw_query,
            }
        })
        .parse_next(i)
}

impl Display for Query {
    /// Formats a [Query] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let query = Query::new("'X', 002@{ 0 | 0 =^ 'O'}")?;
    /// assert_eq!(query.to_string(), "'X', 002@{ 0 | 0 =^ 'O'}");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_query)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Query {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Query {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Fragment {
    Path(Path),
    Literal(BString),
    #[cfg(feature = "unstable")]
    Format(Format),
}

impl Fragment {
    fn query(
        &self,
        record: &RecordRef,
        options: &QueryOptions,
    ) -> Outcome<BString> {
        use Fragment::*;

        match self {
            Literal(lit) => Outcome(vec![vec![lit.clone()]]),
            #[cfg(feature = "unstable")]
            Format(fmt) => Outcome(
                record
                    .format(fmt, &options.into())
                    .map(|e| vec![e])
                    .collect::<Vec<Vec<_>>>(),
            ),
            Path(path) => {
                let fields = record
                    .fields()
                    .iter()
                    .filter(|field| {
                        path.tag_matcher.is_match(field.tag())
                            && path
                                .occurrence_matcher
                                .is_match(field.occurrence())
                    })
                    .filter(|field| {
                        if let Some(ref m) = path.subfield_matcher {
                            m.is_match(
                                field.subfields(),
                                &options.into(),
                            )
                        } else {
                            true
                        }
                    });

                let mut outcome = fields
                    .map(|field| {
                        path.codes
                            .iter()
                            .map(|codes| {
                                field
                                    .subfields()
                                    .iter()
                                    .filter(|subfield| {
                                        codes.contains(subfield.code())
                                    })
                                    .map(|subfield| subfield.value())
                                    .map(|value| {
                                        BString::from(value.as_bytes())
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .map(|values| {
                                if values.is_empty() {
                                    Outcome(vec![vec![BString::from(
                                        "",
                                    )]])
                                } else {
                                    Outcome(
                                        values
                                            .into_iter()
                                            .map(|value| vec![value])
                                            .collect(),
                                    )
                                }
                            })
                            .map(|outcome| {
                                if options.squash {
                                    outcome.squash(&options.separator)
                                } else {
                                    outcome
                                }
                            })
                            .fold(Outcome::default(), |acc, e| acc * e)
                    })
                    .fold(Outcome::default(), |acc, e| acc + e);

                if outcome.is_empty() {
                    outcome = Outcome(vec![
                        std::iter::repeat_n(
                            BString::from(""),
                            path.codes.len(),
                        )
                        .collect(),
                    ]);
                }

                outcome
            }
        }
    }
}

fn parse_fragment(i: &mut &[u8]) -> ModalResult<Fragment> {
    alt((
        parse_path.map(Fragment::Path),
        parse_string.map(|s| Fragment::Literal(s.into())),
        #[cfg(feature = "unstable")]
        parse_format.map(Fragment::Format),
    ))
    .parse_next(i)
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

impl From<QueryOptions> for MatcherOptions {
    fn from(options: QueryOptions) -> Self {
        MatcherOptions::new()
            .strsim_threshold(options.strsim_threshold)
            .case_ignore(options.case_ignore)
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
    #[inline]
    fn from(options: &QueryOptions) -> Self {
        Self::default()
            .strsim_threshold(options.strsim_threshold)
            .case_ignore(options.case_ignore)
    }
}

#[cfg(feature = "unstable")]
impl From<&QueryOptions> for FormatOptions {
    #[inline]
    fn from(options: &QueryOptions) -> Self {
        Self::default()
            .strsim_threshold(options.strsim_threshold)
            .case_ignore(options.case_ignore)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Outcome<T: ToString + Clone>(Vec<Vec<T>>);

impl Outcome<BString> {
    fn squash(self, sep: &str) -> Self {
        let flattened =
            self.0.into_iter().flatten().collect::<Vec<BString>>();

        if flattened.len() > 1
            && !sep.is_empty()
            && flattened
                .iter()
                .any(|item| item.contains_str(sep.as_bytes()))
        {
            eprintln!(
                "WARNING: A subfield value contains \
                      squash separator '{sep}'."
            );
        }

        let mut value = BString::new(vec![]);
        for (i, item) in flattened.iter().enumerate() {
            if i > 0 {
                value.push_str(sep);
            }

            value.push_str(item);
        }

        Self(vec![vec![value]])
    }

    fn merge(self, sep: &str) -> Self {
        let result = self.0.clone().into_iter().reduce(|acc, e| {
            let mut result = Vec::new();

            for i in 0..acc.len() {
                let mut value = acc[i].clone();
                value.push_str(sep);
                value.push_str(&e[i]);
                result.push(value)
            }

            result
        });

        Self(vec![result.unwrap()])
    }
}

impl<T: ToString + Clone> Outcome<T> {
    /// Consumes this [Outcome] returning the underlying data.
    pub fn into_inner(self) -> Vec<Vec<T>> {
        self.0
    }
}

impl<T: ToString + Clone> Deref for Outcome<T> {
    type Target = Vec<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ToString + Clone> Add for Outcome<T> {
    type Output = Outcome<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.0;
        result.extend(rhs.0);
        Self(result)
    }
}

impl<T: ToString + Clone> Mul for Outcome<T> {
    type Output = Outcome<T>;

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
            for y in ys.iter() {
                let mut row = x.clone();
                row.extend(y.clone());
                rows.push(row);
            }
        }

        Self(rows)
    }
}

pub trait QueryExt {
    type Value: ToString + Clone;

    fn query(
        &self,
        query: &Query,
        options: &QueryOptions,
    ) -> Outcome<Self::Value>;
}

impl QueryExt for RecordRef<'_> {
    type Value = BString;

    /// Run the query against the [RecordRef] and return the
    /// corresponding [Outcome].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let record = ByteRecord::from_bytes(b"002@ \x1f0Abvz\x1e\n")?;
    /// let query = Query::new("002@.0")?;
    /// let outcome = record.query(&query, &Default::default());
    /// assert_eq!(outcome.into_inner(), vec![vec!["Abvz"]]);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn query(
        &self,
        query: &Query,
        options: &QueryOptions,
    ) -> Outcome<Self::Value> {
        query
            .fragments
            .iter()
            .map(|fragment| fragment.query(self, options))
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

impl QueryExt for StringRecord<'_> {
    type Value = String;

    /// Run the query against the [StringRecord] and return the
    /// corresponding [Outcome].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let record = ByteRecord::from_bytes(b"002@ \x1f0Abvz\x1e\n")?;
    /// let record = StringRecord::try_from(record)?;
    /// let query = Query::new("002@.0")?;
    /// let outcome = record.query(&query, &Default::default());
    /// assert_eq!(outcome.into_inner(), vec![vec!["Abvz".to_string()]]);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn query(
        &self,
        query: &Query,
        options: &QueryOptions,
    ) -> Outcome<Self::Value> {
        let rows = self.0.query(query, options).into_inner();
        Outcome(
            rows.iter()
                .map(|row| {
                    row.iter().map(ToString::to_string).collect()
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;
    use std::{env, fs};

    use serde_test::{Token, assert_tokens};

    use super::*;
    use crate::ByteRecord;

    type TestResult = anyhow::Result<()>;

    fn ada_lovelace() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let path = std::path::Path::new(&manifest_dir)
                .join("tests/data/ada.dat");
            fs::read_to_string(&path).unwrap().as_bytes().to_vec()
        })
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_query_serde() -> TestResult {
        assert_tokens(&Query::new("003@.0")?, &[Token::Str("003@.0")]);
        assert_tokens(&Query::new("'foo'")?, &[Token::Str("'foo'")]);
        assert_tokens(
            &Query::new("'x', 002@.0")?,
            &[Token::Str("'x', 002@.0")],
        );
        assert_tokens(
            &Query::new("041A/*{ 9 | 9? }")?,
            &[Token::Str("041A/*{ 9 | 9? }")],
        );
        Ok(())
    }

    #[test]
    fn test_query_literal() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        let options = QueryOptions::new();

        let query = Query::new("'foo'")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["foo"]]
        );

        let query = Query::new("'foo','bar'")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["foo", "bar"]]
        );

        Ok(())
    }

    #[test]
    fn test_query_path() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        let options = QueryOptions::new();

        let query = Query::new("003@.0")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["119232022"]]
        );

        let query = Query::new("003@.0, 002@.0")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["119232022", "Tp1"]]
        );

        let query = Query::new("003@.0, 008A.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![
                vec!["119232022", "s"],
                vec!["119232022", "z"],
                vec!["119232022", "f"]
            ]
        );

        let query = Query::new("003@.0, 008X.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["119232022", ""]]
        );

        let query = Query::new("008X.a,008Y.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["", ""]]
        );

        let query = Query::new("008A.a,008B.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![
                vec!["s", "w"],
                vec!["s", "k"],
                vec!["s", "v"],
                vec!["z", "w"],
                vec!["z", "k"],
                vec!["z", "v"],
                vec!["f", "w"],
                vec!["f", "k"],
                vec!["f", "v"],
            ]
        );

        Ok(())
    }

    #[test]
    fn test_query_path_squash() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;

        let options = QueryOptions::new().squash(true);
        let query = Query::new("003@.0, 008A.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["119232022", "s|z|f"],]
        );

        let options = QueryOptions::new().squash(true);
        let query = Query::new("008A.a,008B.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["s|z|f", "w|k|v"]]
        );

        let options = QueryOptions::new().squash(true).separator("+++");
        let query = Query::new("003@.0, 008A.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["119232022", "s+++z+++f"],]
        );

        let data =
            b"012A \x1faX\x1fb1\x1fb2\x1e012A \x1faX\x1fb3\x1e\n";
        let record = ByteRecord::from_bytes(&data)?;
        let options = QueryOptions::new().squash(true);
        let query = Query::new("012A{ a, b }")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["X", "1|2"], vec!["X", "3"]]
        );

        Ok(())
    }

    #[test]
    fn test_query_path_merge() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;

        let options = QueryOptions::new().merge(true);
        let query = Query::new("003@.0, 008A.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["119232022", "s|z|f"],]
        );

        let options = QueryOptions::new().merge(true);
        let query = Query::new("008A.a,008B.a")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["s|z|f", "w|k|v"]]
        );

        let data =
            b"012A \x1faX\x1fb1\x1fb2\x1e012A \x1faX\x1fb3\x1e\n";
        let record = ByteRecord::from_bytes(&data)?;
        let options = QueryOptions::new().merge(true);
        let query = Query::new("012A{ a, b }")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["X|X|X", "1|2|3"]],
        );

        Ok(())
    }

    #[test]
    #[cfg(feature = "unstable")]
    fn test_query_format() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        let options = QueryOptions::new();

        let query = Query::new("003@{ 'https://d-nb.info/gnd/' 0 }")?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec!["https://d-nb.info/gnd/119232022"]]
        );

        let query = Query::new(
            "003@{'https://d-nb.info/gnd/' 0}, 028A{a <$> ', ' d}",
        )?;
        assert_eq!(
            record.query(&query, &options).into_inner(),
            vec![vec![
                "https://d-nb.info/gnd/119232022",
                "Lovelace, Ada King"
            ]]
        );

        Ok(())
    }
}
