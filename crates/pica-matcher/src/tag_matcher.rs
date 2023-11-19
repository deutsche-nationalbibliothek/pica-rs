use std::str::FromStr;

use pica_record::parser::parse_tag;
use pica_record::{Tag, TagRef};
use winnow::combinator::{alt, delimited, fold_repeat, separated_pair};
use winnow::token::one_of;
use winnow::{PResult, Parser};

use crate::ParseMatcherError;

/// A matcher that matches against PICA+ [Tags](`pica_record::Tag`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagMatcher {
    Simple(Tag),
    Pattern([Vec<u8>; 4]),
}

fn parse_fragment(allowed: &[u8], i: &mut &[u8]) -> PResult<Vec<u8>> {
    alt((
        one_of(|c: u8| allowed.contains(&c)).map(|c| vec![c]),
        '.'.value(allowed.to_vec()),
        delimited(
            '[',
            fold_repeat(
                1..,
                alt((
                    separated_pair(
                        one_of(|c| allowed.contains(&c)),
                        '-',
                        one_of(|c| allowed.contains(&c)),
                    )
                    .verify(|(min, max)| min < max)
                    .map(|(min, max)| (min..=max).collect()),
                    one_of(|c| allowed.contains(&c)).map(|c| vec![c]),
                )),
                Vec::new,
                |mut acc, item| {
                    acc.extend(&item);
                    acc
                },
            ),
            ']',
        ),
    ))
    .parse_next(i)
}

#[inline]
fn parse_pattern(i: &mut &[u8]) -> PResult<TagMatcher> {
    let p0 = parse_fragment(b"012", i)?;
    let p1 = parse_fragment(b"0123456789", i)?;
    let p2 = parse_fragment(b"0123456789", i)?;
    let p3 = parse_fragment(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ@", i)?;

    Ok(TagMatcher::Pattern([p0, p1, p2, p3]))
}

#[inline]
fn parse_simple(i: &mut &[u8]) -> PResult<TagMatcher> {
    parse_tag
        .map(|tag| TagMatcher::Simple(Tag::from(tag)))
        .parse_next(i)
}

pub fn parse_tag_matcher(i: &mut &[u8]) -> PResult<TagMatcher> {
    alt((parse_simple, parse_pattern)).parse_next(i)
}

impl TagMatcher {
    /// Create a new tag matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::TagMatcher;
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = TagMatcher::new("003@");
    ///     assert_eq!(matcher, TagRef::new("003@"));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<B: ?Sized + AsRef<[u8]>>(value: &B) -> Self {
        parse_tag_matcher
            .parse(value.as_ref())
            .expect("tag matcher")
    }

    /// Returns `true` if the given tag matches against the matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::TagMatcher;
    /// use pica_record::TagRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = TagMatcher::new("00[3-5]@");
    ///     assert!(matcher.is_match(&TagRef::new("003@")));
    ///     assert!(!matcher.is_match(&TagRef::new("002@")));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, tag: &TagRef) -> bool {
        match self {
            Self::Simple(lhs) => lhs == tag,
            Self::Pattern(pattern) => {
                pattern[0].contains(&tag[0])
                    && pattern[1].contains(&tag[1])
                    && pattern[2].contains(&tag[2])
                    && pattern[3].contains(&tag[3])
            }
        }
    }
}

impl PartialEq<TagMatcher> for TagRef<'_> {
    #[inline]
    fn eq(&self, matcher: &TagMatcher) -> bool {
        matcher.is_match(self)
    }
}

impl<'a> PartialEq<TagRef<'_>> for TagMatcher {
    #[inline]
    fn eq(&self, other: &TagRef<'_>) -> bool {
        self.is_match(other)
    }
}

impl PartialEq<TagMatcher> for &TagRef<'_> {
    #[inline]
    fn eq(&self, matcher: &TagMatcher) -> bool {
        matcher.is_match(self)
    }
}

impl PartialEq<&TagRef<'_>> for TagMatcher {
    #[inline]
    fn eq(&self, other: &&TagRef<'_>) -> bool {
        self.is_match(other)
    }
}

impl FromStr for TagMatcher {
    type Err = ParseMatcherError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_tag_matcher
            .parse(s.as_bytes())
            .map_err(|_| ParseMatcherError::InvalidTagMatcher)
    }
}
