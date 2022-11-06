use std::fmt::{self, Display};

use bstr::{BString, ByteSlice};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, map, value, verify};
use nom::multi::many1;
use nom::sequence::{delimited, terminated, tuple};
use nom::Finish;
use pica_record::parser::{parse_subfield_code, ParseResult};
use pica_record::Subfield;
use regex::bytes::RegexBuilder;
use regex::Regex;
use strsim::normalized_levenshtein;

use crate::common::{
    parse_comparison_op_str, parse_string, ws, ComparisonOp,
};
use crate::{MatcherFlags, ParseMatcherError};

const SUBFILED_CODES: &'static str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// A matcher that matches against PICA+
/// [Subfield](`pica_record::Subfield`).
#[derive(Debug)]
pub struct SubfieldMatcher {
    kind: SubfieldMatcherKind,
    matcher_str: String,
}

#[derive(Debug, PartialEq, Eq)]
enum SubfieldMatcherKind {
    Comparison(ComparisionMatcher),
    Regex(RegexMatcher),
    Exists(Vec<char>),
}

impl SubfieldMatcher {
    /// Create a new subfield matcher.
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
        all_consuming(parse_subfield_matcher_kind)(expr.as_ref())
            .finish()
            .map_err(|_| {
                ParseMatcherError::InvalidSubfieldMatcher(
                    expr.to_string(),
                )
            })
            .map(|(_, kind)| Self {
                matcher_str: expr.to_string(),
                kind,
            })
    }

    /// Returns `true` if the given subfield matches against the
    /// matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::{MatcherFlags, SubfieldMatcher};
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::from_bytes(b"\x1f0abc")?;
    ///     let matcher = SubfieldMatcher::new("0 == 'abc'")?;
    ///     let flags = MatcherFlags::default();
    ///
    ///     assert!(matcher.is_match(&subfield, &flags));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match<T>(
        &self,
        subfield: &Subfield<T>,
        flags: &MatcherFlags,
    ) -> bool
    where
        T: AsRef<[u8]>,
    {
        match &self.kind {
            SubfieldMatcherKind::Comparison(matcher) => {
                matcher.is_match(subfield, flags)
            }
            SubfieldMatcherKind::Exists(codes) => {
                codes.contains(&subfield.code())
            }
            SubfieldMatcherKind::Regex(matcher) => {
                matcher.is_match(subfield, flags)
            }
        }
    }
}

impl Display for SubfieldMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.matcher_str)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ComparisionMatcher {
    codes: Vec<char>,
    op: ComparisonOp,
    value: BString,
}

impl ComparisionMatcher {
    pub fn is_match<T>(
        &self,
        subfield: &Subfield<T>,
        flags: &MatcherFlags,
    ) -> bool
    where
        T: AsRef<[u8]>,
    {
        if !self.codes.contains(&subfield.code()) {
            return false;
        }

        let case_compare = |value: &[u8]| -> bool {
            if flags.case_ignore {
                self.value.to_lowercase() == value.to_lowercase()
            } else {
                self.value == value
            }
        };

        let value = subfield.value().as_ref();

        match self.op {
            ComparisonOp::Eq => case_compare(value),
            ComparisonOp::Ne => !case_compare(value),
            ComparisonOp::StartsWith => {
                if flags.case_ignore {
                    value
                        .to_lowercase()
                        .starts_with(&self.value.to_lowercase())
                } else {
                    value.starts_with(&self.value)
                }
            }
            ComparisonOp::EndsWith => {
                if flags.case_ignore {
                    value
                        .to_lowercase()
                        .ends_with(&self.value.to_lowercase())
                } else {
                    value.ends_with(&self.value)
                }
            }
            ComparisonOp::Similar => {
                let score = if flags.case_ignore {
                    normalized_levenshtein(
                        &self.value.to_string().to_lowercase(),
                        &value.to_str_lossy().to_lowercase(),
                    )
                } else {
                    normalized_levenshtein(
                        &self.value.to_string(),
                        &value.to_str_lossy().to_string(),
                    )
                };

                score > flags.strsim_threshold
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RegexMatcher {
    codes: Vec<char>,
    regex: String,
    invert: bool,
}

impl RegexMatcher {
    pub fn is_match<T>(
        &self,
        subfield: &Subfield<T>,
        flags: &MatcherFlags,
    ) -> bool
    where
        T: AsRef<[u8]>,
    {
        if !self.codes.contains(&subfield.code()) {
            return false;
        }

        let re = RegexBuilder::new(&self.regex)
            .case_insensitive(flags.case_ignore)
            .build()
            .unwrap();

        let result = re.is_match(subfield.value().as_ref());

        if self.invert {
            !result
        } else {
            result
        }
    }
}

fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        delimited(char('['), many1(parse_subfield_code), char(']')),
        many1(parse_subfield_code),
        map(parse_subfield_code, |code| vec![code]),
        value(SUBFILED_CODES.chars().collect(), char('*')),
    ))(i)
}

fn parse_comparision_matcher(
    i: &[u8],
) -> ParseResult<ComparisionMatcher> {
    map(
        tuple((
            ws(parse_subfield_codes),
            ws(parse_comparison_op_str),
            ws(parse_string),
        )),
        |(codes, op, value)| ComparisionMatcher {
            codes,
            op,
            value: BString::from(value),
        },
    )(i)
}

fn parse_regex_matcher(i: &[u8]) -> ParseResult<RegexMatcher> {
    map(
        tuple((
            parse_subfield_codes,
            alt((
                value(false, ws(tag("=~"))),
                value(true, ws(tag("!~"))),
            )),
            verify(parse_string, |x| Regex::new(x).is_ok()),
        )),
        |(codes, invert, regex)| RegexMatcher {
            codes,
            regex,
            invert,
        },
    )(i)
}

fn parse_exists_matcher(i: &[u8]) -> ParseResult<Vec<char>> {
    terminated(ws(parse_subfield_codes), char('?'))(i)
}

fn parse_subfield_matcher_kind(
    i: &[u8],
) -> ParseResult<SubfieldMatcherKind> {
    alt((
        map(parse_comparision_matcher, SubfieldMatcherKind::Comparison),
        map(parse_regex_matcher, SubfieldMatcherKind::Regex),
        map(parse_exists_matcher, SubfieldMatcherKind::Exists),
    ))(i)
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;
    use pica_record::SubfieldRef;

    use super::*;

    #[test]
    fn test_parse_subfield_codes() {
        let expected = vec!['0'];
        assert_done_and_eq!(parse_subfield_codes(b"0"), expected);

        let expected = vec!['0', '1', '2'];
        assert_done_and_eq!(parse_subfield_codes(b"[012]"), expected);

        let expected = vec!['0', '1', '2'];
        assert_done_and_eq!(parse_subfield_codes(b"012"), expected);

        let expected = SUBFILED_CODES.chars().collect::<Vec<char>>();
        assert_done_and_eq!(parse_subfield_codes(b"*"), expected);

        assert_error!(parse_subfield_codes(b"!"));
        assert_error!(parse_subfield_codes(b"[012!]"));
    }

    #[test]
    fn test_parse_comparision_matcher() {
        assert_done_and_eq!(
            parse_comparision_matcher(b"0 == 'abc'"),
            ComparisionMatcher {
                codes: vec!['0'],
                op: ComparisonOp::Eq,
                value: BString::from("abc")
            }
        );

        assert_done_and_eq!(
            parse_comparision_matcher(b"01 != 'abc'"),
            ComparisionMatcher {
                codes: vec!['0', '1'],
                op: ComparisonOp::Ne,
                value: BString::from("abc")
            }
        );

        assert_done_and_eq!(
            parse_comparision_matcher(b"[01] =^ 'abc'"),
            ComparisionMatcher {
                codes: vec!['0', '1'],
                op: ComparisonOp::StartsWith,
                value: BString::from("abc")
            }
        );

        assert_done_and_eq!(
            parse_comparision_matcher(b"9 =$ 'abc'"),
            ComparisionMatcher {
                codes: vec!['9'],
                op: ComparisonOp::EndsWith,
                value: BString::from("abc")
            }
        );
    }

    #[test]
    fn test_comparison_matcher_eq() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("0 == 'abc'")?;
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherFlags::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &MatcherFlags::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherFlags::new().case_ignore(true)
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &MatcherFlags::new().case_ignore(true)
        ));

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_ne() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("0 != 'abc'")?;
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherFlags::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &MatcherFlags::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &MatcherFlags::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherFlags::new().case_ignore(true)
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &MatcherFlags::new().case_ignore(true)
        ));

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_starts_with() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("10 =^ 'ab'")?;
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherFlags::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0Abc")?,
            &MatcherFlags::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0dabc")?,
            &MatcherFlags::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0Abc")?,
            &MatcherFlags::new().case_ignore(true)
        ));

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_ends_with() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("[01] =$ 'ba'")?;
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abba")?,
            &MatcherFlags::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0aBBa")?,
            &MatcherFlags::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0aBBa")?,
            &MatcherFlags::new().case_ignore(true)
        ));

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_similar() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("a =* 'Heike'")?;
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1faHeike")?,
            &MatcherFlags::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1faHeiko")?,
            &MatcherFlags::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1faHeiko")?,
            &MatcherFlags::new().strsim_threshold(0.7)
        ));

        Ok(())
    }

    #[test]
    fn test_exists_matcher() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("0?")?;
        let flags = MatcherFlags::default();
        assert!(matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &flags));
        assert!(!matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1fAabc")?, &flags));

        let matcher = SubfieldMatcher::new("[012]?")?;
        let flags = MatcherFlags::default();
        assert!(matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &flags));
        assert!(matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &flags));
        assert!(matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f2abc")?, &flags));
        assert!(!matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f3abc")?, &flags));

        Ok(())
    }

    #[test]
    fn test_regex_matcher() -> anyhow::Result<()> {
        // regular match
        let matcher = SubfieldMatcher::new("0 =~ '^ab'")?;
        let flags = MatcherFlags::default();

        assert!(matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &flags));
        assert!(!matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f0def")?, &flags));
        assert!(!matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &flags));

        // invert match
        let matcher = SubfieldMatcher::new("0 !~ '^ab'")?;
        let flags = MatcherFlags::default();

        assert!(!matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &flags));
        assert!(matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f0def")?, &flags));
        assert!(!matcher
            .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &flags));

        Ok(())
    }
}
