use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, map, opt};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::Finish;
use pica_record::parser::ParseResult;
use pica_record::Field;

use crate::common::ws;
use crate::occurrence_matcher::{
    parse_occurrence_matcher, OccurrenceMatcher,
};
use crate::subfield_matcher::{
    parse_subfield_matcher, parse_subfield_singleton_matcher, Matcher,
};
use crate::tag_matcher::parse_tag_matcher;
use crate::{
    MatcherOptions, ParseMatcherError, SubfieldMatcher, TagMatcher,
};

#[derive(Debug, PartialEq)]
pub enum FieldMatcher {
    Subfield(TagMatcher, OccurrenceMatcher, SubfieldMatcher),
    Exists(TagMatcher, OccurrenceMatcher),
}

impl FieldMatcher {
    /// Create a new field matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::FieldMatcher;
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = FieldMatcher::new("003@?")?;
    ///
    ///     assert!(matcher.is_match(
    ///         &FieldRef::new("003@", None, vec![('0', "123456789X")]),
    ///         &Default::default()
    ///     ));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(data: &str) -> Result<Self, ParseMatcherError> {
        all_consuming(parse_field_matcher)(data.as_bytes())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidFieldMatcher(data.into())
            })
            .map(|(_, matcher)| matcher)
    }

    /// Returns `true` if the given field matches against the field
    /// matcher.
    pub fn is_match<'a, T: AsRef<[u8]> + 'a>(
        &self,
        fields: impl IntoIterator<Item = &'a Field<T>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        fields.into_iter().any(|field| match self {
            Self::Subfield(t, o, s) => {
                t == field.tag()
                    && *o == field.occurrence()
                    && s.is_match(field.subfields(), options)
            }
            Self::Exists(t, o) => {
                t == field.tag() && *o == field.occurrence()
            }
        })
    }
}

fn parse_field_matcher_exists(i: &[u8]) -> ParseResult<FieldMatcher> {
    map(
        terminated(
            pair(ws(parse_tag_matcher), parse_occurrence_matcher),
            char('?'),
        ),
        |(t, o)| FieldMatcher::Exists(t, o),
    )(i)
}

fn parse_field_matcher_subfield(i: &[u8]) -> ParseResult<FieldMatcher> {
    map(
        tuple((
            parse_tag_matcher,
            parse_occurrence_matcher,
            alt((
                map(
                    pair(
                        opt(alt((char('.'), ws(char('$'))))),
                        parse_subfield_singleton_matcher,
                    ),
                    |(_, matcher)| matcher,
                ),
                preceded(
                    ws(char('{')),
                    cut(terminated(
                        parse_subfield_matcher,
                        ws(char('}')),
                    )),
                ),
            )),
        )),
        |(t, o, s)| FieldMatcher::Subfield(t, o, s),
    )(i)
}

fn parse_field_matcher(i: &[u8]) -> ParseResult<FieldMatcher> {
    alt((parse_field_matcher_exists, parse_field_matcher_subfield))(i)
}
