use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, map};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::Finish;

use crate::common::{ws, ParseResult};
use crate::matcher::{
    parse_occurrence_matcher, parse_subfield_list_matcher,
    parse_subfield_list_matcher_singleton, parse_tag_matcher, MatcherFlags,
    OccurrenceMatcher, SubfieldListMatcher, TagMatcher,
};
use crate::{Error, Field};

#[derive(Debug, PartialEq)]
pub enum FieldMatcher {
    Subield(TagMatcher, OccurrenceMatcher, SubfieldListMatcher),
    Exists(TagMatcher, OccurrenceMatcher),
}

impl FieldMatcher {
    /// Creates a field matcher from a string slice.
    ///
    /// If an invalid field matcher is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::FieldMatcher;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(FieldMatcher::new("012A/*{0? && 0 == 'abc'}").is_ok());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        let data = data.as_ref();

        match all_consuming(parse_field_matcher)(data.as_bytes()).finish() {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(Error::InvalidMatcher(format!(
                "Expected valid field matcher, got '{}'",
                data
            ))),
        }
    }

    /// Returns true, if and only if the given field matches against
    /// the field matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::{FieldMatcher, MatcherFlags};
    /// use pica::Field;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = FieldMatcher::new("012A/*{0? && 0 == 'abc'}")?;
    ///     let field = Field::from_str("012A/01 \x1f0abc\x1e")?;
    ///     assert!(matcher.is_match(&field, &MatcherFlags::default()));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, field: &Field, flags: &MatcherFlags) -> bool {
        match self {
            Self::Subield(tag, occurrence, subfield) => {
                tag.is_match(field.tag())
                    && occurrence.is_match(field.occurrence())
                    && subfield.is_match(field.subfields(), flags)
            }
            Self::Exists(tag, occurrence) => {
                tag.is_match(field.tag())
                    && occurrence.is_match(field.occurrence())
            }
        }
    }
}

fn parse_field_matcher_subfield(i: &[u8]) -> ParseResult<FieldMatcher> {
    map(
        tuple((
            parse_tag_matcher,
            parse_occurrence_matcher,
            alt((
                preceded(char('.'), cut(parse_subfield_list_matcher_singleton)),
                preceded(
                    ws(char('{')),
                    cut(terminated(parse_subfield_list_matcher, ws(char('}')))),
                ),
            )),
        )),
        |(tag, occurrence, subfields)| {
            FieldMatcher::Subield(tag, occurrence, subfields)
        },
    )(i)
}

pub(crate) fn parse_field_matcher_exists(
    i: &[u8],
) -> ParseResult<FieldMatcher> {
    map(
        terminated(
            pair(ws(parse_tag_matcher), parse_occurrence_matcher),
            ws(char('?')),
        ),
        |(t, o)| FieldMatcher::Exists(t, o),
    )(i)
}

pub(crate) fn parse_field_matcher(i: &[u8]) -> ParseResult<FieldMatcher> {
    alt((parse_field_matcher_subfield, parse_field_matcher_exists))(i)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_field_matcher_exists() -> TestResult {
        let matcher = FieldMatcher::new("012A?")?;
        let field = Field::from_str("012A \x1f0abc\x1e")?;
        assert!(matcher.is_match(&field, &MatcherFlags::default()));

        let matcher = FieldMatcher::new("013A?")?;
        let field = Field::from_str("012A \x1f0abc\x1e")?;
        assert!(!matcher.is_match(&field, &MatcherFlags::default()));

        let matcher = FieldMatcher::new("012A/00?")?;
        let field = Field::from_str("012A \x1f0abc\x1e")?;
        assert!(matcher.is_match(&field, &MatcherFlags::default()));

        let matcher = FieldMatcher::new("012A/01?")?;
        let field = Field::from_str("012A/01 \x1f0abc\x1e")?;
        assert!(matcher.is_match(&field, &MatcherFlags::default()));

        let matcher = FieldMatcher::new("012A/01?")?;
        let field = Field::from_str("012A/02 \x1f0abc\x1e")?;
        assert!(!matcher.is_match(&field, &MatcherFlags::default()));

        Ok(())
    }

    #[test]
    fn test_field_matcher_subfield() -> TestResult {
        let matcher = FieldMatcher::new("012A.0 == 'abc'")?;
        let field = Field::from_str("012A \x1f0abc\x1e")?;
        assert!(matcher.is_match(&field, &MatcherFlags::default()));

        let matcher = FieldMatcher::new("012A{0 == 'abc'}")?;
        let field = Field::from_str("012A \x1f0abc\x1e")?;
        assert!(matcher.is_match(&field, &MatcherFlags::default()));

        let matcher = FieldMatcher::new("012A/01{0 == 'abc'}")?;
        let field = Field::from_str("012A/01 \x1f0abc\x1e")?;
        assert!(matcher.is_match(&field, &MatcherFlags::default()));

        let matcher = FieldMatcher::new("012A{0 == 'abc' && 9?}")?;
        let field = Field::from_str("012A \x1f0abc\x1f9123\x1e")?;
        assert!(matcher.is_match(&field, &MatcherFlags::default()));

        Ok(())
    }
}
