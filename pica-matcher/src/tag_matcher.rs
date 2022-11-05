use std::fmt::{self, Display};

use nom::branch::alt;
use nom::character::complete::{char, one_of};
use nom::combinator::{all_consuming, map, value};
use nom::multi::fold_many1;
use nom::sequence::{delimited, separated_pair};
use nom::Finish;
use pica_record::parser::{parse_tag, ParseResult};
use pica_record::{Tag, TagMut};

use crate::ParseMatcherError;

/// A matcher that matches against PICA+ [Tags](`pica_record::Tag`).
#[derive(Debug)]
pub struct TagMatcher {
    kind: TagMatcherKind,
    matcher_str: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TagMatcherKind {
    Simple(TagMut),
    Pattern([Vec<char>; 4]),
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
    ///     let matcher = TagMatcher::new("003@")?;
    ///     assert_eq!(matcher, TagRef::new("003@"));
    ///
    ///     # assert!(TagMatcher::new("003!").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T>(expr: T) -> Result<Self, ParseMatcherError>
    where
        T: AsRef<[u8]> + Display,
    {
        all_consuming(parse_tag_matcher_kind)(expr.as_ref())
            .finish()
            .map_err(|_| ParseMatcherError::InvalidTagMatcher)
            .map(|(_, tag)| Self {
                matcher_str: expr.to_string(),
                kind: tag,
            })
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
    ///     let matcher = TagMatcher::new("003@")?;
    ///     assert!(matcher.is_match(&TagRef::new("003@")));
    ///     assert!(!matcher.is_match(&TagRef::new("002@")));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match<T: AsRef<[u8]>>(&self, tag: &Tag<T>) -> bool {
        match &self.kind {
            TagMatcherKind::Simple(lhs) => lhs == tag,
            TagMatcherKind::Pattern(pattern) => {
                pattern[0].contains(&(tag[0] as char))
                    && pattern[1].contains(&(tag[1] as char))
                    && pattern[2].contains(&(tag[2] as char))
                    && pattern[3].contains(&(tag[3] as char))
            }
        }
    }
}

impl PartialEq for TagMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<T: AsRef<[u8]>> PartialEq<TagMatcher> for Tag<T> {
    #[inline]
    fn eq(&self, matcher: &TagMatcher) -> bool {
        matcher.is_match(self)
    }
}

impl<T: AsRef<[u8]>> PartialEq<Tag<T>> for TagMatcher {
    #[inline]
    fn eq(&self, tag: &Tag<T>) -> bool {
        self.is_match(tag)
    }
}

impl Display for TagMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.matcher_str)
    }
}

#[inline]
fn parse_fragment<'a>(
    allowed: &'a str,
    i: &'a [u8],
) -> ParseResult<'a, Vec<char>> {
    alt((
        map(one_of(allowed), |c| vec![c]),
        delimited(
            char('['),
            fold_many1(
                alt((
                    map(
                        separated_pair(
                            one_of(allowed),
                            char('-'),
                            one_of(allowed),
                        ),
                        |(min, max)| (min..=max).collect(),
                    ),
                    map(one_of(allowed), |c| vec![c]),
                )),
                Vec::new,
                |mut acc, item| {
                    acc.extend(&item);
                    acc
                },
            ),
            char(']'),
        ),
        value(allowed.chars().collect(), char('.')),
    ))(i)
}

#[inline]
fn parse_pattern(i: &[u8]) -> ParseResult<TagMatcherKind> {
    let (i, p0) = parse_fragment("012", i)?;
    let (i, p1) = parse_fragment("0123456789", i)?;
    let (i, p2) = parse_fragment("0123456789", i)?;
    let (i, p3) = parse_fragment("ABCDEFGHIJKLMNOPQRSTUVWXYZ@", i)?;

    Ok((i, TagMatcherKind::Pattern([p0, p1, p2, p3])))
}

#[inline]
fn parse_simple(i: &[u8]) -> ParseResult<TagMatcherKind> {
    map(parse_tag, |tag| {
        TagMatcherKind::Simple(TagMut::from_unchecked(tag))
    })(i)
}

fn parse_tag_matcher_kind(i: &[u8]) -> ParseResult<TagMatcherKind> {
    alt((parse_simple, parse_pattern))(i)
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;
    use pica_record::{TagMut, TagRef};

    use super::*;

    #[test]
    fn test_parse_fragment() {
        assert_done_and_eq!(parse_fragment("012", b"0"), vec!['0']);
        assert_done_and_eq!(
            parse_fragment("012", b"[02]"),
            vec!['0', '2']
        );
        assert_done_and_eq!(
            parse_fragment("012", b"."),
            vec!['0', '1', '2']
        );

        assert_error!(parse_fragment("012", b"3"));
        assert_error!(parse_fragment("012", b"[03]"));
    }

    #[test]
    fn test_parse_simple() {
        assert_done_and_eq!(
            parse_simple(b"003@"),
            TagMatcherKind::Simple(TagMut::from_unchecked("003@")),
        );

        assert_error!(parse_simple(b"003!"));
    }

    #[test]
    fn test_parse_pattern() {
        assert_done_and_eq!(
            parse_pattern(b"00[23]@"),
            TagMatcherKind::Pattern([
                vec!['0'],
                vec!['0'],
                vec!['2', '3'],
                vec!['@']
            ])
        );

        assert_done_and_eq!(
            parse_pattern(b"00[2-5]@"),
            TagMatcherKind::Pattern([
                vec!['0'],
                vec!['0'],
                vec!['2', '3', '4', '5'],
                vec!['@']
            ])
        );
        assert_done_and_eq!(
            parse_pattern(b"00[13-57]@"),
            TagMatcherKind::Pattern([
                vec!['0'],
                vec!['0'],
                vec!['1', '3', '4', '5', '7'],
                vec!['@']
            ])
        );

        assert_done_and_eq!(
            parse_pattern(b"00[5-2]@"),
            TagMatcherKind::Pattern([
                vec!['0'],
                vec!['0'],
                vec![],
                vec!['@']
            ])
        );
    }

    #[test]
    fn test_parse_tag_matcher_kind() {
        assert_done_and_eq!(
            parse_tag_matcher_kind(b"003@"),
            TagMatcherKind::Simple(TagMut::from_unchecked("003@")),
        );

        assert_done_and_eq!(
            parse_tag_matcher_kind(b"00[2-4][A@]"),
            TagMatcherKind::Pattern([
                vec!['0'],
                vec!['0'],
                vec!['2', '3', '4'],
                vec!['A', '@']
            ])
        );

        assert_done_and_eq!(
            parse_tag_matcher_kind(b"00[4-2][A@]"),
            TagMatcherKind::Pattern([
                vec!['0'],
                vec!['0'],
                vec![],
                vec!['A', '@']
            ])
        );

        assert_done_and_eq!(
            parse_tag_matcher_kind(b".12A"),
            TagMatcherKind::Pattern([
                vec!['0', '1', '2'],
                vec!['1'],
                vec!['2'],
                vec!['A']
            ])
        );

        assert_done_and_eq!(
            parse_tag_matcher_kind(b"00[2-49][A@]"),
            TagMatcherKind::Pattern([
                vec!['0'],
                vec!['0'],
                vec!['2', '3', '4', '9'],
                vec!['A', '@']
            ])
        );

        assert_done_and_eq!(
            parse_tag_matcher_kind(b"...."),
            TagMatcherKind::Pattern([
                ('0'..='2').collect(),
                ('0'..='9').collect(),
                ('0'..='9').collect(),
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ@".chars().collect(),
            ])
        );

        assert_done!(parse_tag_matcher_kind(b"[0-2][0-9][0-9][A-Z@]"));
        assert_done!(parse_tag_matcher_kind(b"0[0-9]2A"));
        assert_done!(parse_tag_matcher_kind(b"012A"));

        assert_error!(parse_tag_matcher_kind(b"[1-9]12A"));
        assert_error!(parse_tag_matcher_kind(b"[4-5]12A"));
        assert_error!(parse_tag_matcher_kind(b"[34]12A"));
        assert_error!(parse_tag_matcher_kind(b"003!"));
    }

    #[test]
    fn test_tag_matcher_new() -> anyhow::Result<()> {
        assert_eq!(
            TagMatcher::new("003@")?,
            TagMatcher {
                matcher_str: "003@".to_string(),
                kind: TagMatcherKind::Simple(TagMut::from_bytes(
                    b"003@"
                )?)
            }
        );

        assert_eq!(
            TagMatcher::new("00[23]@")?,
            TagMatcher {
                matcher_str: "00[23]@".to_string(),
                kind: TagMatcherKind::Pattern([
                    vec!['0'],
                    vec!['0'],
                    vec!['2', '3'],
                    vec!['@']
                ])
            }
        );

        assert_eq!(
            TagMatcher::new("00[2-3]@")?,
            TagMatcher {
                matcher_str: "00[2-3]@".to_string(),
                kind: TagMatcherKind::Pattern([
                    vec!['0'],
                    vec!['0'],
                    vec!['2', '3'],
                    vec!['@']
                ])
            }
        );

        Ok(())
    }

    #[test]
    fn test_tag_matcher_is_match() -> anyhow::Result<()> {
        let matcher = TagMatcher::new("003@")?;
        assert!(!matcher.is_match(&TagRef::from_bytes(b"002@")?));
        assert!(matcher.is_match(&TagRef::from_bytes(b"003@")?));

        let matcher = TagMatcher::new("00[23]@")?;
        assert!(matcher.is_match(&TagRef::from_bytes(b"002@")?));
        assert!(matcher.is_match(&TagRef::from_bytes(b"003@")?));

        Ok(())
    }

    #[test]
    fn test_tag_matcher_to_string() -> anyhow::Result<()> {
        let matcher = TagMatcher::new("003@")?;
        assert_eq!(matcher.to_string(), "003@".to_string());

        let matcher = TagMatcher::new("00[2-3]@")?;
        assert_eq!(matcher.to_string(), "00[2-3]@".to_string());

        let matcher = TagMatcher::new("00[23]@")?;
        assert_eq!(matcher.to_string(), "00[23]@".to_string());

        Ok(())
    }

    #[test]
    fn test_tag_matcher_partial_eq() -> anyhow::Result<()> {
        let matcher = TagMatcher::new("003@")?;

        let tag_ref = TagRef::from_bytes(b"003@")?;
        assert_eq!(tag_ref, matcher);
        assert_eq!(matcher, tag_ref);

        let tag_ref = TagRef::from_bytes(b"002@")?;
        assert_ne!(tag_ref, matcher);
        assert_ne!(matcher, tag_ref);

        assert_eq!(
            TagMatcher::new("00[2-3]@")?,
            TagMatcher::new("00[23]@")?,
        );

        Ok(())
    }
}
