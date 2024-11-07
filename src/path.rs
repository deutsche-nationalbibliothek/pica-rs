use std::fmt::{self, Display};
use std::sync::LazyLock;

use bstr::{BStr, ByteSlice};
use smallvec::SmallVec;
use winnow::combinator::{alt, delimited, opt, preceded, separated};
use winnow::{PResult, Parser};

use crate::matcher::occurrence::parse_occurrence_matcher;
use crate::matcher::subfield::parser::parse_subfield_matcher;
use crate::matcher::subfield::SubfieldMatcher;
use crate::matcher::tag::parse_tag_matcher;
use crate::matcher::{MatcherOptions, OccurrenceMatcher, TagMatcher};
use crate::parser::{parse_subfield_codes, ws};
use crate::primitives::{FieldRef, RecordRef, SubfieldCode};
use crate::StringRecord;

/// An error that can occur when parsing a path expression.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ParsePathError(pub(crate) String);

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    codes: Vec<SmallVec<[SubfieldCode; 4]>>,
    raw_path: String,
}

impl Path {
    /// Creates a new [Path].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// path expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let _path = Path::new("041A/*{ (9, a) | 9? }")?;
    /// let _path = Path::new("003@.0")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(path: &str) -> Result<Self, ParsePathError> {
        parse_path.parse(path.as_bytes()).map_err(|_| {
            ParsePathError(format!("invalid path '{path}'"))
        })
    }
}

fn parse_path_simple(i: &mut &[u8]) -> PResult<Path> {
    ws((
        parse_tag_matcher,
        parse_occurrence_matcher,
        preceded('.', parse_subfield_codes),
    ))
    .with_taken()
    .map(|((t, o, c), raw_path)| {
        let raw_path = raw_path.to_str().unwrap().to_string();
        Path {
            tag_matcher: t,
            occurrence_matcher: o,
            codes: vec![c],
            subfield_matcher: None,
            raw_path,
        }
    })
    .parse_next(i)
}

fn parse_path_curly(i: &mut &[u8]) -> PResult<Path> {
    ws((
        parse_tag_matcher,
        parse_occurrence_matcher,
        delimited(
            ws('{'),
            (
                alt((
                    separated(1.., parse_subfield_codes, ws(',')),
                    delimited(
                        ws('('),
                        separated(1.., parse_subfield_codes, ws(',')),
                        ws(')'),
                    ),
                )),
                opt(preceded(ws('|'), parse_subfield_matcher)),
            ),
            ws('}'),
        ),
    ))
    .with_taken()
    .map(|((t, o, (c, m)), raw_path)| {
        let raw_path = raw_path.to_str().unwrap().to_string();
        Path {
            tag_matcher: t,
            occurrence_matcher: o,
            codes: c,
            subfield_matcher: m,
            raw_path,
        }
    })
    .parse_next(i)
}

#[inline]
fn parse_path(i: &mut &[u8]) -> PResult<Path> {
    alt((parse_path_simple, parse_path_curly)).parse_next(i)
}

impl Display for Path {
    /// Formats a [Path] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let path = Path::new("002@{ 0 | 0 =^ 'O'}")?;
    /// assert_eq!(path.to_string(), "002@{ 0 | 0 =^ 'O'}");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_path)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

pub trait PathExt {
    type Value: ?Sized;

    /// Returns an iterator over the path values.
    ///
    /// Note that tuple values are flattened to a single list.
    fn path(
        &self,
        path: &Path,
        options: &MatcherOptions,
    ) -> impl Iterator<Item = &Self::Value>;

    /// Returns the first value of the path, or `None` if the path is
    /// empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let record =
    ///     ByteRecord::from_bytes(b"042A \x1fa12.2p\x1f18p\x1e\n")?;
    /// let record = StringRecord::try_from(record)?;
    ///
    /// assert_eq!(
    ///     record.first(&Path::new("042A.a")?, &Default::default()),
    ///     Some("12.2p")
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn first(
        &self,
        path: &Path,
        options: &MatcherOptions,
    ) -> Option<&Self::Value> {
        self.path(path, options).next()
    }

    /// Returns the PICA production number of the record.
    ///
    /// # Panics
    ///
    /// This function panics if the record doesn't have a ppn. This
    /// should never happen, because a PPN is automatically assigned
    /// when a new data record is saved.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let record = ByteRecord::from_bytes(b"003@ \x1f0118540238\x1e\n")?;
    /// assert_eq!(record.ppn(), "118540238");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn ppn(&self) -> &Self::Value {
        static PATH: LazyLock<Path> =
            LazyLock::new(|| Path::new("003@.0").unwrap());

        self.first(&PATH, &Default::default()).unwrap()
    }
}

impl PathExt for RecordRef<'_> {
    type Value = BStr;

    /// Returns the path values as an iterator over byte slices.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let record = ByteRecord::from_bytes(b"002@ \x1f0Abvz\x1e\n")?;
    /// let path = Path::new("002@.0")?;
    /// let values: Vec<_> =
    ///     record.path(&path, &Default::default()).collect();
    /// assert_eq!(values, vec!["Abvz"]);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn path(
        &self,
        path: &Path,
        options: &MatcherOptions,
    ) -> impl Iterator<Item = &Self::Value> {
        self.fields()
            .iter()
            .filter(|field| {
                let retval = path.tag_matcher.is_match(field.tag())
                    && path
                        .occurrence_matcher
                        .is_match(field.occurrence());

                if let Some(ref matcher) = path.subfield_matcher {
                    retval
                        && matcher.is_match(field.subfields(), options)
                } else {
                    retval
                }
            })
            .flat_map(FieldRef::subfields)
            .filter_map(|subfield| {
                if path
                    .codes
                    .iter()
                    .any(|codes| codes.contains(subfield.code()))
                {
                    Some(subfield.value().as_bstr())
                } else {
                    None
                }
            })
    }
}

impl PathExt for StringRecord<'_> {
    type Value = str;

    /// Returns the path values as an iterator over string slices.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let record = ByteRecord::from_bytes(b"002@ \x1f0Abvz\x1e\n")?;
    /// let record = StringRecord::try_from(record)?;
    /// let path = Path::new("002@.0")?;
    /// let values: Vec<_> =
    ///     record.path(&path, &Default::default()).collect();
    /// assert_eq!(values, vec!["Abvz"]);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn path(
        &self,
        path: &Path,
        options: &MatcherOptions,
    ) -> impl Iterator<Item = &Self::Value> {
        self.0.path(path, options).map(|value| {
            // SAFETY: It's safe to call unwrap, because a StringRecord
            // guarantees valid UTF-8 strings and it's not necessary to
            // validate the values again.
            value.to_str().unwrap()
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;
    use std::{env, fs};

    use serde_test::{assert_tokens, Token};

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
    fn test_path_serde() -> TestResult {
        assert_tokens(&Path::new("003@.0")?, &[Token::Str("003@.0")]);
        assert_tokens(
            &Path::new("041A/*{ 9 | 9? }")?,
            &[Token::Str("041A/*{ 9 | 9? }")],
        );
        Ok(())
    }

    #[test]
    fn test_parse_path_simple() -> TestResult {
        assert_eq!(
            parse_path_simple.parse(b"003@.0").unwrap(),
            Path {
                tag_matcher: TagMatcher::new("003@")?,
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: None,
                codes: vec![SmallVec::from_vec(vec![
                    SubfieldCode::new('0')?
                ])],
                raw_path: "003@.0".to_string(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_path_curly() -> TestResult {
        assert_eq!(
            parse_path_curly
                .parse(b"045E{ (e, f) | E == 'm' }")
                .unwrap(),
            Path {
                tag_matcher: TagMatcher::new("045E")?,
                occurrence_matcher: OccurrenceMatcher::None,
                subfield_matcher: Some(SubfieldMatcher::new(
                    "E == 'm'"
                )?),
                codes: vec![
                    SmallVec::from_vec(vec![SubfieldCode::new('e')?]),
                    SmallVec::from_vec(vec![SubfieldCode::new('f')?]),
                ],
                raw_path: "045E{ (e, f) | E == 'm' }".to_string(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_path_new() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        let options = MatcherOptions::default();
        let path = Path::new("008B.a")?;

        let values: Vec<_> = record.path(&path, &options).collect();
        assert_eq!(values, vec!["w", "k", "v"]);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_path_new_panic() {
        let _ = Path::new("003@.!").unwrap();
    }

    #[test]
    fn test_byte_record_path() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        let options = MatcherOptions::default();
        let path = Path::new("008A.a")?;

        let values: Vec<&BStr> = record.path(&path, &options).collect();
        assert_eq!(values, vec!["s", "z", "f"]);
        Ok(())
    }

    #[test]
    fn test_string_record_path() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        let record = StringRecord::try_from(record)?;
        let options = MatcherOptions::default();

        let path = Path::new("008A.a")?;

        let values: Vec<&str> = record.path(&path, &options).collect();
        assert_eq!(values, vec!["s", "z", "f"]);
        Ok(())
    }

    #[test]
    fn test_path_with_predicate() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        let options = MatcherOptions::default();
        let path = Path::new("007N{ 0 | a == 'swd'}")?;

        let values: Vec<&BStr> = record.path(&path, &options).collect();
        assert_eq!(values, vec!["4370325-2"]);
        Ok(())
    }

    #[test]
    fn test_path_first() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        let options = MatcherOptions::default();
        let path = Path::new("007[KN]{0 | a in ['pnd','swd']}")?;
        assert_eq!(record.first(&path, &options).unwrap(), "172642531");
        Ok(())
    }

    #[test]
    fn test_path_ppn() -> TestResult {
        let data = ada_lovelace();
        let record = ByteRecord::from_bytes(&data)?;
        assert_eq!(record.ppn(), "119232022");
        Ok(())
    }
}
