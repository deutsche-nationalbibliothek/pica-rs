use std::fmt::{self, Display};

use bstr::{BString, ByteSlice};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{
    all_consuming, cut, map, map_res, opt, value, verify,
};
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::Finish;
use pica_record::parser::{parse_subfield_code, ParseResult};
use pica_record::Subfield;
use regex::bytes::RegexBuilder;
use regex::Regex;
use strsim::normalized_levenshtein;

use crate::common::{
    parse_comparison_op_str, parse_comparison_op_usize, parse_string,
    ws, ComparisonOp,
};
use crate::{MatcherOptions, ParseMatcherError};

const SUBFILED_CODES: &str =
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
    Cardinality(CardinalityMatcher),
    Regex(RegexMatcher),
    Exists(Vec<char>),
    In(InMatcher),
}

impl SubfieldMatcher {
    /// Create a new subfield matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::SubfieldMatcher;
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SubfieldMatcher::new("0 == 'abc'")?;
    ///     assert_eq!(matcher, SubfieldRef::from_bytes(b"\x1f0abc")?);
    ///
    ///     # assert!(SubfieldMatcher::new("# != 'def'").is_err());
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
    /// use pica_matcher::{MatcherOptions, SubfieldMatcher};
    /// use pica_record::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let subfield = SubfieldRef::from_bytes(b"\x1f0abc")?;
    ///     let matcher = SubfieldMatcher::new("0 == 'abc'")?;
    ///     let options = MatcherOptions::default();
    ///
    ///     assert!(matcher.is_match(&subfield, &options));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match<'a, T>(
        &self,
        subfields: impl IntoIterator<Item = &'a Subfield<T>>,
        flags: &MatcherOptions,
    ) -> bool
    where
        T: AsRef<[u8]> + 'a,
    {
        let mut subfields = subfields.into_iter();

        match &self.kind {
            SubfieldMatcherKind::Comparison(matcher) => subfields
                .any(|subfield| matcher.is_match(subfield, flags)),
            SubfieldMatcherKind::Exists(codes) => subfields
                .any(|subfield| codes.contains(&subfield.code())),
            SubfieldMatcherKind::Regex(matcher) => subfields
                .any(|subfield| matcher.is_match(subfield, flags)),
            SubfieldMatcherKind::In(matcher) => subfields
                .any(|subfield| matcher.is_match(subfield, flags)),
            SubfieldMatcherKind::Cardinality(matcher) => {
                let count = subfields
                    .filter(|s| s.code() == matcher.code)
                    .count();

                match matcher.op {
                    ComparisonOp::Eq => count == matcher.value,
                    ComparisonOp::Ne => count != matcher.value,
                    ComparisonOp::Ge => count >= matcher.value,
                    ComparisonOp::Gt => count > matcher.value,
                    ComparisonOp::Le => count <= matcher.value,
                    ComparisonOp::Lt => count < matcher.value,
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl Display for SubfieldMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.matcher_str)
    }
}

impl<T: AsRef<[u8]>> PartialEq<Subfield<T>> for SubfieldMatcher {
    /// Tests if `subfield` matches against `self` using default
    /// [`MatcherOptions`].
    fn eq(&self, subfield: &Subfield<T>) -> bool {
        self.is_match(subfield, &MatcherOptions::default())
    }
}

impl<T: AsRef<[u8]>> PartialEq<SubfieldMatcher> for Subfield<T> {
    /// Tests if `matcher` matches against `self` using default
    /// [`MatcherOptions`].
    fn eq(&self, matcher: &SubfieldMatcher) -> bool {
        matcher.is_match(self, &MatcherOptions::default())
    }
}

/// A matcher that uses basic comparison operators like `==` or `!=`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ComparisionMatcher {
    codes: Vec<char>,
    op: ComparisonOp,
    value: BString,
}

impl ComparisionMatcher {
    /// Returns `true` if the given subfield matches against the
    /// comparison matcher.
    fn is_match<T>(
        &self,
        subfield: &Subfield<T>,
        options: &MatcherOptions,
    ) -> bool
    where
        T: AsRef<[u8]>,
    {
        if !self.codes.contains(&subfield.code()) {
            return false;
        }

        let case_compare = |value: &[u8]| -> bool {
            if options.case_ignore {
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
                if options.case_ignore {
                    value
                        .to_lowercase()
                        .starts_with(&self.value.to_lowercase())
                } else {
                    value.starts_with(&self.value)
                }
            }
            ComparisonOp::EndsWith => {
                if options.case_ignore {
                    value
                        .to_lowercase()
                        .ends_with(&self.value.to_lowercase())
                } else {
                    value.ends_with(&self.value)
                }
            }
            ComparisonOp::Similar => {
                let score = if options.case_ignore {
                    normalized_levenshtein(
                        &self.value.to_string().to_lowercase(),
                        &value.to_str_lossy().to_lowercase(),
                    )
                } else {
                    normalized_levenshtein(
                        &self.value.to_string(),
                        &value.to_str_lossy(),
                    )
                };

                score > options.strsim_threshold
            }
            _ => unreachable!(),
        }
    }
}

/// A matcher that uses regular expressions.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RegexMatcher {
    codes: Vec<char>,
    regex: String,
    invert: bool,
}

impl RegexMatcher {
    /// Returns `true` if given subfield matches against the regular
    /// expression.
    fn is_match<T>(
        &self,
        subfield: &Subfield<T>,
        options: &MatcherOptions,
    ) -> bool
    where
        T: AsRef<[u8]>,
    {
        if !self.codes.contains(&subfield.code()) {
            return false;
        }

        let re = RegexBuilder::new(&self.regex)
            .case_insensitive(options.case_ignore)
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

/// A matcher that checks if the subfield value is contained in a list.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InMatcher {
    codes: Vec<char>,
    values: Vec<BString>,
    invert: bool,
}

impl InMatcher {
    /// Returns `true` if the given subfield value is in the values
    /// list.
    fn is_match<T>(
        &self,
        subfield: &Subfield<T>,
        options: &MatcherOptions,
    ) -> bool
    where
        T: AsRef<[u8]>,
    {
        if !self.codes.contains(&subfield.code()) {
            return false;
        }

        let result = self.values.iter().any(|value: &BString| {
            if options.case_ignore {
                value.to_lowercase()
                    == subfield.value().as_ref().to_lowercase()
            } else {
                value == subfield.value().as_ref()
            }
        });

        if self.invert {
            !result
        } else {
            result
        }
    }
}

/// A matcher that checks the cardinality of a subfield.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CardinalityMatcher {
    code: char,
    op: ComparisonOp,
    value: usize,
}

/// Parse a single or list of subfield codes.
fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        delimited(char('['), many1(parse_subfield_code), char(']')),
        many1(parse_subfield_code),
        map(parse_subfield_code, |code| vec![code]),
        value(SUBFILED_CODES.chars().collect(), char('*')),
    ))(i)
}

/// Parse the `ComparisionMatcher` kind.
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

/// Parse the `RegexMatcher` kind.
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

/// Parse the `InMatcher` kind.
fn parse_in_matcher(i: &[u8]) -> ParseResult<InMatcher> {
    map(
        tuple((
            parse_subfield_codes,
            map(opt(ws(tag("not"))), |not| not.is_some()),
            ws(tag("in")),
            delimited(
                ws(char('[')),
                separated_list1(
                    ws(char(',')),
                    map(parse_string, BString::from),
                ),
                ws(char(']')),
            ),
        )),
        |(codes, invert, _, values)| InMatcher {
            codes,
            values,
            invert,
        },
    )(i)
}

fn parse_exists_matcher(i: &[u8]) -> ParseResult<Vec<char>> {
    terminated(ws(parse_subfield_codes), char('?'))(i)
}

fn parse_cardinality_matcher(
    i: &[u8],
) -> ParseResult<CardinalityMatcher> {
    map(
        preceded(
            char('#'),
            cut(tuple((
                ws(parse_subfield_code),
                ws(parse_comparison_op_usize),
                map_res(digit1, |s| {
                    std::str::from_utf8(s).unwrap().parse::<usize>()
                }),
            ))),
        ),
        |(code, op, value)| CardinalityMatcher { code, op, value },
    )(i)
}

/// Parse a `SubfieldMatcherKind` matcher.
fn parse_subfield_matcher_kind(
    i: &[u8],
) -> ParseResult<SubfieldMatcherKind> {
    alt((
        map(parse_comparision_matcher, SubfieldMatcherKind::Comparison),
        map(parse_regex_matcher, SubfieldMatcherKind::Regex),
        map(parse_in_matcher, SubfieldMatcherKind::In),
        map(parse_exists_matcher, SubfieldMatcherKind::Exists),
        map(
            parse_cardinality_matcher,
            SubfieldMatcherKind::Cardinality,
        ),
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
    fn test_parse_regex_matcher() {
        assert_done_and_eq!(
            parse_regex_matcher(b"0 =~ '^abc'"),
            RegexMatcher {
                codes: vec!['0'],
                regex: "^abc".into(),
                invert: false,
            }
        );

        assert_done_and_eq!(
            parse_regex_matcher(b"0 !~ '^abc'"),
            RegexMatcher {
                codes: vec!['0'],
                regex: "^abc".into(),
                invert: true,
            }
        );
    }

    #[test]
    fn test_comparison_matcher_partial_eq() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("0 == 'abc'")?;
        assert_eq!(matcher, SubfieldRef::from_bytes(b"\x1f0abc")?);
        assert_eq!(SubfieldRef::from_bytes(b"\x1f0abc")?, matcher);

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_eq() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("0 == 'abc'")?;
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherOptions::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &MatcherOptions::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherOptions::new().case_ignore(true)
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &MatcherOptions::new().case_ignore(true)
        ));

        // multiple subfields
        let matcher = SubfieldMatcher::new("0 == 'abc'")?;
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0def")?,
                &SubfieldRef::from_bytes(b"\x1f0abc")?
            ],
            &MatcherOptions::default()
        ));

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_ne() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("0 != 'abc'")?;
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherOptions::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &MatcherOptions::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &MatcherOptions::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherOptions::new().case_ignore(true)
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &MatcherOptions::new().case_ignore(true)
        ));

        // multiple subfields
        let matcher = SubfieldMatcher::new("0 != 'abc'")?;
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0def")?,
                &SubfieldRef::from_bytes(b"\x1f0abc")?
            ],
            &MatcherOptions::default()
        ));

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_starts_with() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("10 =^ 'ab'")?;
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &MatcherOptions::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0Abc")?,
            &MatcherOptions::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0dabc")?,
            &MatcherOptions::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0Abc")?,
            &MatcherOptions::new().case_ignore(true)
        ));

        // multiple subfields
        let matcher = SubfieldMatcher::new("10 =^ 'ab'")?;
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
                &SubfieldRef::from_bytes(b"\x1f0cba")?,
            ],
            &MatcherOptions::default()
        ));

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_ends_with() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("[01] =$ 'ba'")?;
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abba")?,
            &MatcherOptions::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0aBBa")?,
            &MatcherOptions::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0aBBa")?,
            &MatcherOptions::new().case_ignore(true)
        ));

        // multiple subfields
        let matcher = SubfieldMatcher::new("[01] =$ 'ba'")?;
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0baab")?,
                &SubfieldRef::from_bytes(b"\x1f0abba")?
            ],
            &MatcherOptions::default()
        ));

        Ok(())
    }

    #[test]
    fn test_comparison_matcher_similar() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("a =* 'Heike'")?;
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1faHeike")?,
            &MatcherOptions::default()
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1faHeiko")?,
            &MatcherOptions::default()
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1faHeiko")?,
            &MatcherOptions::new().strsim_threshold(0.7)
        ));

        // multiple subfields
        let matcher = SubfieldMatcher::new("a =* 'Heike'")?;

        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1faHeike")?,
                &SubfieldRef::from_bytes(b"\x1faHeino")?,
            ],
            &MatcherOptions::default()
        ));

        assert!(!matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1faHeiko")?,
                &SubfieldRef::from_bytes(b"\x1faHeino")?,
            ],
            &MatcherOptions::default()
        ));

        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1faHeiko")?,
                &SubfieldRef::from_bytes(b"\x1faHeino")?,
            ],
            &MatcherOptions::new().strsim_threshold(0.7)
        ));

        Ok(())
    }

    #[test]
    fn test_exists_matcher() -> anyhow::Result<()> {
        let matcher = SubfieldMatcher::new("0?")?;
        let options = MatcherOptions::default();
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1fAabc")?,
            &options
        ));

        let matcher = SubfieldMatcher::new("[012]?")?;
        let options = MatcherOptions::default();
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f1abc")?,
            &options
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f2abc")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f3abc")?,
            &options
        ));

        // multiple subfields
        let matcher = SubfieldMatcher::new("0?")?;
        let options = MatcherOptions::default();

        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1fA123")?,
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
            ],
            &options
        ));

        assert!(!matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1fA123")?,
                &SubfieldRef::from_bytes(b"\x1f1abc")?,
            ],
            &options
        ));

        Ok(())
    }

    #[test]
    fn test_regex_matcher() -> anyhow::Result<()> {
        // regular match
        let matcher = SubfieldMatcher::new("0 =~ '^ab'")?;
        let options = MatcherOptions::default();

        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f1abc")?,
            &options
        ));

        // invert match
        let matcher = SubfieldMatcher::new("0 !~ '^ab'")?;
        let options = MatcherOptions::default();

        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f1abc")?,
            &options
        ));

        // multiple subfields
        let matcher = SubfieldMatcher::new("0 =~ '^ab'")?;
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0def")?,
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
            ],
            &MatcherOptions::default()
        ));

        Ok(())
    }

    #[test]
    fn test_in_matcher() -> anyhow::Result<()> {
        // regular match
        let matcher = SubfieldMatcher::new("0 in ['abc', 'def']")?;
        let options = MatcherOptions::default();

        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &options
        ));

        // case ignore
        let matcher = SubfieldMatcher::new("0 in ['abc', 'def']")?;
        let options = MatcherOptions::new().case_ignore(true);

        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &options
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0hij")?,
            &options
        ));

        // inverted match
        let matcher = SubfieldMatcher::new("0 not in ['abc', 'def']")?;
        let options = MatcherOptions::default();

        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &options
        ));
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0ABC")?,
            &options
        ));

        // multiple subfields
        let matcher = SubfieldMatcher::new("0 in ['abc', 'def']")?;
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0xyz")?,
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
            ],
            &MatcherOptions::default()
        ));

        Ok(())
    }

    #[test]
    fn test_cardinality_matcher() -> anyhow::Result<()> {
        // Equal, `==`
        let matcher = SubfieldMatcher::new("#0 == 1")?;
        let options = MatcherOptions::default();

        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(!matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
                &SubfieldRef::from_bytes(b"\x1f0def")?,
            ],
            &options
        ));

        let matcher = SubfieldMatcher::new("#0 == 0")?;
        let options = MatcherOptions::default();

        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));

        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1fXabc")?,
            &options
        ));

        // Not Equal, `!=`
        let matcher = SubfieldMatcher::new("#0 != 1")?;
        let options = MatcherOptions::default();

        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
                &SubfieldRef::from_bytes(b"\x1f0def")?,
            ],
            &options
        ));

        // Greater than or equal, `>=`
        let matcher = SubfieldMatcher::new("#0 >= 2")?;
        let options = MatcherOptions::default();

        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
                &SubfieldRef::from_bytes(b"\x1f0def")?,
            ],
            &options
        ));

        // Greater than, `>`
        let matcher = SubfieldMatcher::new("#0 > 1")?;
        let options = MatcherOptions::default();

        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
                &SubfieldRef::from_bytes(b"\x1f0def")?,
            ],
            &options
        ));

        // Less than or equal, `<=`
        let matcher = SubfieldMatcher::new("#0 <= 1")?;
        let options = MatcherOptions::default();

        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(!matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
                &SubfieldRef::from_bytes(b"\x1f0def")?,
            ],
            &options
        ));

        // Less than or equal, `<=`
        let matcher = SubfieldMatcher::new("#0 < 2")?;
        let options = MatcherOptions::default();

        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &options
        ));
        assert!(!matcher.is_match(
            vec![
                &SubfieldRef::from_bytes(b"\x1f0abc")?,
                &SubfieldRef::from_bytes(b"\x1f0def")?,
            ],
            &options
        ));

        let matcher = SubfieldMatcher::new("#0 < 1")?;
        let options = MatcherOptions::default();
        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1fXabc")?,
            &options
        ));

        Ok(())
    }
}
