use std::str::FromStr;

use pica_record_v1::parser::parse_tag;
use pica_record_v1::{Tag, TagRef};
use winnow::combinator::{alt, delimited, repeat, separated_pair};
use winnow::token::one_of;
use winnow::{PResult, Parser};

use crate::ParseMatcherError;

/// A matcher that matches against PICA+ [Tags](`pica_record_v1::Tag`).
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
            repeat(
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
            )
            .fold(Vec::new, |mut acc, item| {
                acc.extend(&item);
                acc
            }),
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

#[inline]
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
    /// use pica_record_v1::TagRef;
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
    /// use pica_record_v1::TagRef;
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

impl PartialEq<TagRef<'_>> for TagMatcher {
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! pattern {
        ($p0:expr, $p1:expr, $p2:expr, $p3:expr) => {
            TagMatcher::Pattern([
                $p0.as_bytes().to_vec(),
                $p1.as_bytes().to_vec(),
                $p2.as_bytes().to_vec(),
                $p3.as_bytes().to_vec(),
            ])
        };
    }

    #[test]
    fn parse_simple() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    super::parse_simple.parse($input).unwrap(),
                    $expected
                );
            };
        }

        parse_success!(b"003@", TagMatcher::Simple(Tag::new("003@")));
        parse_success!(b"101@", TagMatcher::Simple(Tag::new("101@")));
        parse_success!(b"203@", TagMatcher::Simple(Tag::new("203@")));

        assert!(super::parse_simple.parse(b"003@.0").is_err());
        assert!(super::parse_simple.parse(b"!03@").is_err());
    }

    #[test]
    fn parse_pattern() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    super::parse_pattern.parse($input).unwrap(),
                    $expected
                );
            };
        }

        parse_success!(b"003@", pattern!("0", "0", "3", "@"));
        parse_success!(b"[02]03@", pattern!("02", "0", "3", "@"));
        parse_success!(b".03@", pattern!("012", "0", "3", "@"));
        parse_success!(b"0.3@", pattern!("0", "0123456789", "3", "@"));
        parse_success!(b"00.@", pattern!("0", "0", "0123456789", "@"));
        parse_success!(b"0[2-4]1A", pattern!("0", "234", "1", "A"));
        parse_success!(b"0[2-46]1A", pattern!("0", "2346", "1", "A"));

        parse_success!(
            b"003.",
            pattern!("0", "0", "3", "ABCDEFGHIJKLMNOPQRSTUVWXYZ@")
        );

        parse_success!(
            b"0[2-456-8]1A",
            pattern!("0", "2345678", "1", "A")
        );

        parse_success!(
            b"....",
            pattern!(
                "012",
                "0123456789",
                "0123456789",
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ@"
            )
        );

        assert!(super::parse_pattern.parse(b"00[3-1]@").is_err());
        assert!(super::parse_pattern.parse(b"00[3-3]@").is_err());
    }

    #[test]
    fn parse_tag_matcher() {
        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    super::parse_tag_matcher.parse($input).unwrap(),
                    $expected
                );
            };
        }

        parse_success!(b"003@", TagMatcher::Simple(Tag::new("003@")));
        parse_success!(b"0[2-46]1A", pattern!("0", "2346", "1", "A"));
    }
}
