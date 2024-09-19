use std::fmt::{self, Display};

use winnow::Parser;

use super::parse::parse_tag_matcher;
use super::ParseMatcherError;
use crate::primitives::{Tag, TagRef};

/// A matcher that matches against a TagRef.
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
