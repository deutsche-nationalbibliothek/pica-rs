use std::str::FromStr;

use bstr::{BStr, ByteSlice};
use pica_matcher::parser::{
    parse_occurrence_matcher, parse_tag_matcher,
};
use pica_matcher::subfield_matcher::parse_subfield_matcher;
use pica_matcher::{
    MatcherOptions, OccurrenceMatcher, SubfieldMatcher, TagMatcher,
};
use pica_record::parser::parse_subfield_code;
use pica_record::{FieldRef, RecordRef, SubfieldCode};
#[cfg(feature = "serde")]
use serde::Deserialize;
use thiserror::Error;
use winnow::ascii::multispace0;
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, separated, separated_pair,
};
use winnow::error::ParserError;
use winnow::prelude::*;
use winnow::stream::{AsChar, Stream, StreamIsPartial};

const SUBFIELD_CODES: &str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Error)]
#[error("invalid path expression, got `{0}`")]
pub struct ParsePathError(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    codes: Vec<Vec<SubfieldCode>>,
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
    pub fn new<T: ?Sized + AsRef<[u8]>>(data: &T) -> Self {
        Self::try_from(data.as_ref()).expect("valid path expression.")
    }

    /// Returns the list of codes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_path::Path;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let path = Path::new("003@.0");
    ///     assert_eq!(path.codes(), &[vec!['0']]);
    ///     Ok(())
    /// }
    /// ```
    pub fn codes(&self) -> &Vec<Vec<SubfieldCode>> {
        &self.codes
    }

    /// Returns the flat list of codes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_path::Path;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let path = Path::new("003@.0");
    ///     assert_eq!(path.codes_flat(), &['0']);
    ///     Ok(())
    /// }
    /// ```
    pub fn codes_flat(&self) -> Vec<SubfieldCode> {
        self.codes.clone().into_iter().flatten().collect()
    }

    /// Returns the tag matcher of the path.
    pub fn tag_matcher(&self) -> &TagMatcher {
        &self.tag_matcher
    }

    /// Returns the occurrence matcher of the path.
    pub fn occurrence_matcher(&self) -> &OccurrenceMatcher {
        &self.occurrence_matcher
    }

    /// Returns the subfield matcher of the path.
    pub fn subfield_matcher(&self) -> Option<&SubfieldMatcher> {
        self.subfield_matcher.as_ref()
    }
}

impl TryFrom<&[u8]> for Path {
    type Error = ParsePathError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_path.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParsePathError(value)
        })
    }
}

impl FromStr for Path {
    type Err = ParsePathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// Strip whitespaces from the beginning and end.
pub(crate) fn ws<I, O, E: ParserError<I>, F>(
    mut inner: F,
) -> impl Parser<I, O, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar + Clone,
    F: Parser<I, O, E>,
{
    move |i: &mut I| {
        let _ = multispace0.parse_next(i)?;
        let o = inner.parse_next(i);
        let _ = multispace0.parse_next(i)?;

        o
    }
}

#[inline]
fn parse_subfield_code_range(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    separated_pair(parse_subfield_code, '-', parse_subfield_code)
        .verify(|(min, max)| min < max)
        .map(|(min, max)| {
            (min.as_byte()..=max.as_byte())
                .map(SubfieldCode::from_unchecked)
                .collect()
        })
        .parse_next(i)
}

#[inline]
fn parse_subfield_code_single(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    parse_subfield_code.map(|code| vec![code]).parse_next(i)
}

#[inline]
fn parse_subfield_code_list(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    delimited(
        '[',
        repeat(
            1..,
            alt((
                parse_subfield_code_range,
                parse_subfield_code_single,
            )),
        )
        .fold(Vec::new, |mut acc: Vec<_>, item| {
            acc.extend_from_slice(&item);
            acc
        }),
        ']',
    )
    .parse_next(i)
}

#[inline]
fn parse_subfield_codes(i: &mut &[u8]) -> PResult<Vec<SubfieldCode>> {
    alt((
        parse_subfield_code_list,
        parse_subfield_code_single,
        '*'.value(
            SUBFIELD_CODES
                .chars()
                .map(|code| SubfieldCode::new(code).unwrap())
                .collect(),
        ),
    ))
    .parse_next(i)
}

fn parse_path_simple(i: &mut &[u8]) -> PResult<Path> {
    ws((
        parse_tag_matcher,
        parse_occurrence_matcher,
        preceded('.', parse_subfield_codes),
    ))
    .map(|(t, o, c)| Path {
        tag_matcher: t,
        occurrence_matcher: o,
        subfield_matcher: None,
        codes: vec![c],
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
    .map(|(t, o, (c, m))| Path {
        tag_matcher: t,
        occurrence_matcher: o,
        subfield_matcher: m,
        codes: c,
    })
    .parse_next(i)
}

pub fn parse_path(i: &mut &[u8]) -> PResult<Path> {
    alt((parse_path_simple, parse_path_curly)).parse_next(i)
}

pub trait PathExt {
    fn path(&self, path: &Path, options: &MatcherOptions)
        -> Vec<&BStr>;

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
    ///     assert_eq!(record.idn(), Some(b"123456789X".as_bstr()));
    ///
    ///     let record = ByteRecord::from_bytes(b"002@ \x1f0Olfo\x1e\n")?;
    ///     assert_eq!(record.idn(), None);
    ///     Ok(())
    /// }
    /// ```
    fn idn(&self) -> Option<&BStr> {
        self.path(&Path::new("003@.0"), &Default::default())
            .first()
            .copied()
    }

    /// Returns the first value (converted to string) of the path
    /// expression.
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
    ///     assert_eq!(
    ///         record.first("003@.0"),
    ///         Some("123456789X".to_string())
    ///     );
    ///
    ///     let record = ByteRecord::from_bytes(b"002@ \x1f0Olfo\x1e\n")?;
    ///     assert_eq!(record.first("003@.0"), None);
    ///     Ok(())
    /// }
    /// ```
    fn first<P: AsRef<[u8]>>(&self, path: P) -> Option<String> {
        self.path(&Path::new(&path), &Default::default())
            .first()
            .map(ToString::to_string)
    }
}

impl<'a> PathExt for RecordRef<'a> {
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
    fn path(
        &self,
        path: &Path,
        options: &MatcherOptions,
    ) -> Vec<&BStr> {
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
            .flat_map(FieldRef::subfields)
            .filter_map(|subfield| {
                if path.codes_flat().contains(subfield.code()) {
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
        let s: String = Deserialize::deserialize(deserializer)?;
        Path::try_from(s.as_bytes()).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_subfield_code_single() {
        use super::parse_subfield_code_single;

        assert_eq!(
            parse_subfield_code_single.parse(b"a").unwrap(),
            vec!['a']
        );
    }

    #[test]
    fn parse_subfield_code_range() {
        use super::parse_subfield_code_range;

        assert_eq!(
            parse_subfield_code_range.parse(b"a-c").unwrap(),
            vec!['a', 'b', 'c']
        );

        assert!(parse_subfield_code_range.parse(b"a-a").is_err());
        assert!(parse_subfield_code_range.parse(b"c-a").is_err());
        assert!(parse_subfield_code_range.parse(b"a").is_err());
    }

    #[test]
    fn parse_subfield_codes() {
        use super::{parse_subfield_codes, SUBFIELD_CODES};

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    parse_subfield_codes.parse($input).unwrap(),
                    $expected
                )
            };
        }

        parse_success!(b"[a-cx]", vec!['a', 'b', 'c', 'x']);
        parse_success!(b"[a-c]", vec!['a', 'b', 'c']);
        parse_success!(b"a", vec!['a']);
        parse_success!(
            b"*",
            SUBFIELD_CODES.chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn parse_path_simple() {
        macro_rules! parse_success {
            ($input:expr) => {
                assert!(super::parse_path_simple.parse($input).is_ok())
            };
        }

        parse_success!(b"021A/*.[a-cx]");
        parse_success!(b"021A.[a-cx]");
        parse_success!(b"021A/*.[a-c]");
        parse_success!(b"021A.[a-c]");
        parse_success!(b"021A/*.a");
        parse_success!(b"..../*.*");
        parse_success!(b"021A.a");
        parse_success!(b"021A.*");
    }

    #[test]
    fn parse_path_curly() {
        macro_rules! parse_success {
            ($input:expr) => {
                assert!(super::parse_path_curly.parse($input).is_ok())
            };
        }

        parse_success!(b"021A/*{ [a-cx] }");
        parse_success!(b"021A/*{ [a-cx], y }");
        parse_success!(b"021A/*{ ([a-cx], y) }");
        parse_success!(b"021A/*{ ([a-cx], y) | y? }");
        parse_success!(b"021A/*{ * | y? }");
        parse_success!(b"021A{ [a-cx] }");
        parse_success!(b"021A/*{[a-c]}");
        parse_success!(b"021A{[a-c]}");
        parse_success!(b"021A/*{a}");
        parse_success!(b"021A{a}");
        parse_success!(b"021A{*}");
    }
}
