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

/// An error that can occur when parsing PICA+ records.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ParsePathError(pub(crate) String);

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    codes: Vec<SmallVec<[SubfieldCode; 4]>>,
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
    .map(|(t, o, c)| Path {
        tag_matcher: t,
        occurrence_matcher: o,
        codes: vec![c],
        subfield_matcher: None,
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
        codes: c,
        subfield_matcher: m,
    })
    .parse_next(i)
}

#[inline]
fn parse_path(i: &mut &[u8]) -> PResult<Path> {
    alt((parse_path_simple, parse_path_curly)).parse_next(i)
}

pub trait PathExt {
    type Value: ?Sized;

    fn path(
        &self,
        path: &Path,
        options: &MatcherOptions,
    ) -> impl Iterator<Item = &Self::Value>;
}

impl PathExt for RecordRef<'_> {
    type Value = BStr;

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
