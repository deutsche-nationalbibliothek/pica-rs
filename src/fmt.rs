use std::cell::RefCell;
use std::ops::RangeTo;

use bstr::ByteSlice;
use smallvec::SmallVec;
use winnow::ascii::digit1;
use winnow::combinator::{
    alt, delimited, empty, opt, preceded, repeat, separated, terminated,
};
use winnow::error::ParserError;
use winnow::{PResult, Parser};

use crate::matcher::occurrence::parse_occurrence_matcher;
use crate::matcher::subfield::parser::parse_subfield_matcher;
use crate::matcher::subfield::SubfieldMatcher;
use crate::matcher::tag::parse_tag_matcher;
use crate::matcher::{OccurrenceMatcher, TagMatcher};
use crate::parser::{parse_string, parse_subfield_codes, ws};
use crate::primitives::SubfieldCode;

/// An error that can occur when parsing a format expression.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("{0}")]
pub struct ParseFormatError(pub(crate) String);

#[derive(Debug, Clone, PartialEq)]
pub struct Format {
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    raw_format: String,
    fragments: Fragments,
}

impl Format {
    /// Creates a new [Format].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// format expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    ///
    /// let _fmt = Format::new("028[A@]{ a }")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn new(fmt: &str) -> Result<Self, ParseFormatError> {
        parse_format.parse(fmt.as_bytes()).map_err(|_| {
            ParseFormatError(format!("invalid format '{fmt}'"))
        })
    }
}

fn parse_format(i: &mut &[u8]) -> PResult<Format> {
    (
        parse_tag_matcher,
        parse_occurrence_matcher,
        delimited(
            ws('{'),
            (
                parse_fragments,
                opt(preceded(ws('|'), parse_subfield_matcher)),
            ),
            ws('}'),
        ),
    )
        .with_taken()
        .map(|((t, o, (f, s)), r)| {
            let raw_format = r.to_str().unwrap().trim().to_string();
            Format {
                tag_matcher: t,
                occurrence_matcher: o,
                subfield_matcher: s,
                fragments: f,
                raw_format,
            }
        })
        .parse_next(i)
}

#[derive(Debug, Clone, PartialEq)]
enum Fragments {
    Group(Group),
    Value(Value),
    List(List),
}

fn parse_fragments(i: &mut &[u8]) -> PResult<Fragments> {
    alt((
        ws(parse_list).map(Fragments::List),
        ws(parse_group).map(Fragments::Group),
        ws(parse_value).map(Fragments::Value),
    ))
    .parse_next(i)
}

#[derive(Debug, Clone, PartialEq)]
struct Group {
    fragments: Box<Fragments>,
    bounds: RangeTo<usize>,
    modifier: Modifier,
}

thread_local! {
    pub static FORMAT_FRAGMENT_GROUP_LEVEL: RefCell<u32> = const { RefCell::new(0) };
}

fn group_level_inc(i: &mut &[u8]) -> PResult<()> {
    FORMAT_FRAGMENT_GROUP_LEVEL.with(|level| {
        *level.borrow_mut() += 1;
        if *level.borrow() >= 32 {
            Err(winnow::error::ErrMode::from_error_kind(
                i,
                winnow::error::ErrorKind::Many,
            ))
        } else {
            Ok(())
        }
    })
}

fn group_level_dec() {
    FORMAT_FRAGMENT_GROUP_LEVEL.with(|level| {
        *level.borrow_mut() -= 1;
    })
}

fn group_level_reset() {
    FORMAT_FRAGMENT_GROUP_LEVEL.with(|level| *level.borrow_mut() = 0);
}

fn parse_group(i: &mut &[u8]) -> PResult<Group> {
    (
        terminated(ws('('), group_level_inc),
        parse_modifier,
        parse_fragments,
        ws(')').map(|_| group_level_dec()),
        alt((
            preceded(
                "..",
                digit1
                    .verify_map(|s: &[u8]| s.to_str().ok())
                    .verify_map(|s: &str| s.parse::<usize>().ok()),
            ),
            "..".value(usize::MAX),
            empty.value(usize::MAX),
        )),
    )
        .map(|(_, modifier, fragments, _, end)| {
            group_level_reset();
            Group {
                fragments: Box::new(fragments),
                bounds: RangeTo { end },
                modifier: modifier.unwrap_or_default(),
            }
        })
        .parse_next(i)
}

#[derive(Debug, Clone, PartialEq)]
struct Value {
    codes: SmallVec<[SubfieldCode; 4]>,
    prefix: Option<String>,
    suffix: Option<String>,
    bounds: RangeTo<usize>,
}

fn parse_value(i: &mut &[u8]) -> PResult<Value> {
    (
        opt(ws(parse_string).map(|s| {
            // SAFETY: `parse_string` returns a valid string and
            // therefore it's safe to unwrap the result.
            s.to_str().unwrap().to_string()
        })),
        parse_subfield_codes,
        alt((
            preceded(
                "..",
                digit1
                    .verify_map(|s: &[u8]| s.to_str().ok())
                    .verify_map(|s: &str| s.parse::<usize>().ok()),
            ),
            "..".value(usize::MAX),
            empty.value(1),
        )),
        opt(ws(parse_string).map(|s| {
            // SAFETY: `parse_string` returns a valid string and
            // therefore it's safe to unwrap the result.
            s.to_str().unwrap().to_string()
        })),
    )
        .map(|(prefix, codes, end, suffix)| Value {
            prefix,
            codes,
            suffix,
            bounds: RangeTo { end },
        })
        .parse_next(i)
}

#[derive(Debug, Clone, PartialEq)]
enum List {
    AndThen(Vec<Fragments>),
    Cons(Vec<Fragments>),
}

fn parse_list(i: &mut &[u8]) -> PResult<List> {
    alt((parse_list_cons, parse_list_and_then)).parse_next(i)
}

fn parse_list_cons(i: &mut &[u8]) -> PResult<List> {
    separated(
        2..,
        alt((
            parse_list_and_then.map(Fragments::List),
            parse_group.map(Fragments::Group),
            parse_value.map(Fragments::Value),
        )),
        ws("<*>"),
    )
    .map(List::Cons)
    .parse_next(i)
}

fn parse_list_and_then(i: &mut &[u8]) -> PResult<List> {
    separated(
        2..,
        alt((
            parse_group.map(Fragments::Group),
            parse_value.map(Fragments::Value),
        )),
        ws("<$>"),
    )
    .map(List::AndThen)
    .parse_next(i)
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Modifier {
    lowercase: bool,
    uppercase: bool,
    remove_ws: bool,
    trim: bool,
}

impl Modifier {
    pub(crate) fn lowercase(&mut self, yes: bool) -> &mut Self {
        self.lowercase = yes;
        self
    }

    pub(crate) fn uppercase(&mut self, yes: bool) -> &mut Self {
        self.uppercase = yes;
        self
    }

    pub(crate) fn remove_ws(&mut self, yes: bool) -> &mut Self {
        self.remove_ws = yes;
        self
    }

    pub(crate) fn trim(&mut self, yes: bool) -> &mut Self {
        self.trim = yes;
        self
    }
}

fn parse_modifier(i: &mut &[u8]) -> PResult<Option<Modifier>> {
    opt(preceded(
        '?',
        repeat(1.., alt(('L', 'U', 'T', 'W'))).map(|codes: Vec<_>| {
            let mut modifier = Modifier::default();
            if codes.contains(&'L') {
                modifier.lowercase(true);
            }

            if codes.contains(&'U') {
                modifier.uppercase(true);
            }

            if codes.contains(&'W') {
                modifier.remove_ws(true);
            }

            if codes.contains(&'T') {
                modifier.trim(true);
            }

            modifier
        }),
    ))
    .parse_next(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn test_parse_modifier() -> TestResult {
        assert_eq!(
            parse_modifier.parse(b"?L").unwrap().unwrap(),
            Modifier {
                lowercase: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?U").unwrap().unwrap(),
            Modifier {
                uppercase: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?W").unwrap().unwrap(),
            Modifier {
                remove_ws: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?T").unwrap().unwrap(),
            Modifier {
                trim: true,
                ..Default::default()
            }
        );

        assert_eq!(
            parse_modifier.parse(b"?LUWT").unwrap().unwrap(),
            Modifier {
                lowercase: true,
                uppercase: true,
                remove_ws: true,
                trim: true,
            }
        );

        Ok(())
    }
}
