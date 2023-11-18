use pica_record::parser::parse_tag;
use pica_record::Tag;
use winnow::combinator::{alt, delimited, fold_repeat, separated_pair};
use winnow::token::one_of;
use winnow::{PResult, Parser};

/// A matcher that matches against PICA+ [Tags](`pica_record::Tag`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagMatcher<'a> {
    Simple(Tag<'a>),
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
fn parse_pattern<'a>(i: &mut &'a [u8]) -> PResult<TagMatcher<'a>> {
    let p0 = parse_fragment(b"012", i)?;
    let p1 = parse_fragment(b"0123456789", i)?;
    let p2 = parse_fragment(b"0123456789", i)?;
    let p3 = parse_fragment(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ@", i)?;

    Ok(TagMatcher::Pattern([p0, p1, p2, p3]))
}

#[inline]
fn parse_simple<'a>(i: &mut &'a [u8]) -> PResult<TagMatcher<'a>> {
    parse_tag.map(TagMatcher::Simple).parse_next(i)
}

pub fn parse_tag_matcher<'a>(
    i: &mut &'a [u8],
) -> PResult<TagMatcher<'a>> {
    alt((parse_simple, parse_pattern)).parse_next(i)
}

impl<'a> TagMatcher<'a> {
    /// Create a new tag matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::TagMatcher;
    /// use pica_record::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = TagMatcher::new("003@");
    ///     assert_eq!(matcher, Tag::new("003@"));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<B: ?Sized + AsRef<[u8]>>(value: &'a B) -> Self {
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
    /// use pica_record::Tag;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = TagMatcher::new("00[3-5]@");
    ///     assert!(matcher.is_match(&Tag::new("003@")));
    ///     assert!(!matcher.is_match(&Tag::new("002@")));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, tag: &Tag) -> bool {
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

impl<'a> PartialEq<TagMatcher<'a>> for Tag<'_> {
    #[inline]
    fn eq(&self, matcher: &TagMatcher) -> bool {
        matcher.is_match(self)
    }
}

impl<'a> PartialEq<Tag<'_>> for TagMatcher<'a> {
    #[inline]
    fn eq(&self, other: &Tag<'_>) -> bool {
        self.is_match(other)
    }
}

impl<'a> PartialEq<TagMatcher<'a>> for &Tag<'_> {
    #[inline]
    fn eq(&self, matcher: &TagMatcher) -> bool {
        matcher.is_match(self)
    }
}

impl<'a> PartialEq<&Tag<'_>> for TagMatcher<'a> {
    #[inline]
    fn eq(&self, other: &&Tag<'_>) -> bool {
        self.is_match(other)
    }
}
