//! Matcher that can be applied on a list of [TagRef].

use std::fmt::{self, Display};

use bstr::ByteSlice;
use winnow::combinator::{alt, delimited, repeat, separated_pair};
use winnow::error::ParserError;
use winnow::prelude::*;
use winnow::token::one_of;

use super::ParseMatcherError;
use crate::primitives::parse::parse_tag_ref;
use crate::primitives::{Tag, TagRef};

/// A matcher that matches against a [TagRef].
#[derive(Debug, Clone, PartialEq)]
pub enum TagMatcher {
    Tag(Tag),
    Pattern([Vec<u8>; 4], String),
}

impl TagMatcher {
    /// Creates a new [TagMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid tag
    /// matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::TagMatcher;
    ///
    /// let _matcher = TagMatcher::new("041[A@]")?;
    /// let _matcher = TagMatcher::new("003@")?;
    /// let _matcher = TagMatcher::new("00.@")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_tag_matcher.parse(matcher.as_bytes()).map_err(|_| {
            ParseMatcherError(format!(
                "invalid tag matcher '{matcher}'"
            ))
        })
    }

    /// Returns `true` if the given tag matches against the matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::TagMatcher;
    /// use pica_record::primitives::TagRef;
    ///
    /// let matcher = TagMatcher::new("00[3-5]@")?;
    /// assert!(!matcher.is_match(&TagRef::new("002@")?));
    /// assert!(matcher.is_match(&TagRef::new("003@")?));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_match(&self, tag: &TagRef) -> bool {
        match self {
            Self::Tag(lhs, ..) => lhs == tag,
            Self::Pattern(pattern, ..) => {
                pattern[0].contains(&tag[0])
                    && pattern[1].contains(&tag[1])
                    && pattern[2].contains(&tag[2])
                    && pattern[3].contains(&tag[3])
            }
        }
    }
}

impl Display for TagMatcher {
    /// Formats the tag matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::TagMatcher;
    ///
    /// let matcher = TagMatcher::new("00[3-5]@")?;
    /// assert_eq!(matcher.to_string(), "00[3-5]@");
    ///
    /// let matcher = TagMatcher::new("003@")?;
    /// assert_eq!(matcher.to_string(), "003@");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pattern(_, raw_data) => write!(f, "{raw_data}"),
            Self::Tag(tag) => write!(f, "{tag}"),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for TagMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TagMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

#[inline]
fn parse_tag_matcher_tag(i: &mut &[u8]) -> PResult<TagMatcher> {
    parse_tag_ref
        .map(Tag::from)
        .map(TagMatcher::Tag)
        .parse_next(i)
}

fn parse_tag_matcher_pattern_fragment<'a, E: ParserError<&'a [u8]>>(
    allowed: &[u8],
) -> impl Parser<&'a [u8], Vec<u8>, E> + '_ {
    move |i: &mut &'a [u8]| {
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
                        one_of(|c| allowed.contains(&c))
                            .map(|c| vec![c]),
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
}

#[inline]
fn parse_tag_matcher_pattern(i: &mut &[u8]) -> PResult<TagMatcher> {
    (
        parse_tag_matcher_pattern_fragment(b"012"),
        parse_tag_matcher_pattern_fragment(b"0123456789"),
        parse_tag_matcher_pattern_fragment(b"0123456789"),
        parse_tag_matcher_pattern_fragment(
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ@",
        ),
    )
        .with_taken()
        .map(|((p0, p1, p2, p3), raw_data)| {
            let raw_data = raw_data.to_str().unwrap().to_string();
            TagMatcher::Pattern([p0, p1, p2, p3], raw_data)
        })
        .parse_next(i)
}

fn parse_tag_matcher(i: &mut &[u8]) -> PResult<TagMatcher> {
    alt((parse_tag_matcher_tag, parse_tag_matcher_pattern))
        .parse_next(i)
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn test_tag_matcher_serde() -> TestResult {
        let matcher = TagMatcher::new("003@")?;
        assert_tokens(&matcher, &[Token::Str("003@")]);

        let matcher = TagMatcher::new(".0[2-3][A@]")?;
        assert_tokens(&matcher, &[Token::Str(".0[2-3][A@]")]);

        Ok(())
    }

    #[test]
    fn test_parse_tag_matcher() -> TestResult {
        macro_rules! parse_success {
            ($i:expr, $o:expr) => {
                assert_eq!(
                    parse_tag_matcher.parse($i.as_bytes()).unwrap(),
                    $o
                );
            };
        }

        parse_success!("003@", TagMatcher::Tag(Tag::new("003@")?));
        parse_success!("002@", TagMatcher::Tag(Tag::new("002@")?));

        parse_success!(
            ".0[2-4]@",
            TagMatcher::Pattern(
                [
                    vec![b'0', b'1', b'2'],
                    vec![b'0'],
                    vec![b'2', b'3', b'4'],
                    vec![b'@']
                ],
                ".0[2-4]@".to_string()
            )
        );

        parse_success!(
            "00[23]@",
            TagMatcher::Pattern(
                [vec![b'0'], vec![b'0'], vec![b'2', b'3'], vec![b'@']],
                "00[23]@".to_string()
            )
        );

        Ok(())
    }
}
