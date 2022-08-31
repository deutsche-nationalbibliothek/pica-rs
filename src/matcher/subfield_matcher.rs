use std::fmt;

use bstr::{BString, ByteSlice};
use nom::Finish;
use regex::bytes::RegexBuilder;
use regex::Regex;
use strsim::normalized_levenshtein;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, map, opt, value, verify};
use nom::multi::{many1, separated_list1};
use nom::sequence::{preceded, terminated, tuple};

use crate::common::{parse_string, ws, ParseResult};
use crate::matcher::{parse_comparison_op_bstring, ComparisonOp, MatcherFlags};
use crate::subfield::parse_subfield_code;

use crate::{Error, Subfield};

macro_rules! maybe_lowercase {
    ($value:expr, $flag:expr) => {
        if $flag {
            $value.to_lowercase()
        } else {
            $value
        }
    };
}

/// A subfield matcher.
#[derive(Debug, PartialEq, Eq)]
pub enum SubfieldMatcher {
    Comparison(Vec<char>, ComparisonOp, BString),
    Exists(Vec<char>),
    In(Vec<char>, Vec<BString>, bool),
    Regex(Vec<char>, String, bool),
}

fn fmt_codes(codes: &Vec<char>) -> String {
    let result = String::from_iter(codes);
    if result.len() > 1 {
        format!("[{}]", result)
    } else {
        result
    }
}

impl fmt::Display for SubfieldMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Comparison(codes, op, value) => {
                write!(f, "{} {} '{}'", fmt_codes(codes), op, value)
            }
            Self::Exists(codes) => write!(f, "{}?", fmt_codes(codes)),
            Self::In(codes, values, invert) => {
                let values: String = values
                    .iter()
                    .map(|s| format!("'{}'", s))
                    .collect::<Vec<String>>()
                    .join(", ");

                if *invert {
                    write!(f, "{} not in [{}]", fmt_codes(codes), values)
                } else {
                    write!(f, "{} in [{}]", fmt_codes(codes), values)
                }
            }
            Self::Regex(codes, regex, invert) => {
                if *invert {
                    write!(f, "{} !~ '{}'", fmt_codes(codes), regex)
                } else {
                    write!(f, "{} =~ '{}'", fmt_codes(codes), regex)
                }
            }
        }
    }
}

impl SubfieldMatcher {
    /// Creates a subfield matcher from a string slice.
    ///
    /// If an invalid subfield matcher is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::SubfieldMatcher;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(SubfieldMatcher::new("0 == 'abc'").is_ok());
    ///     assert!(SubfieldMatcher::new("! == 'ac'").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        let data = data.as_ref();

        match all_consuming(parse_subfield_matcher)(data.as_bytes()).finish() {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(Error::InvalidMatcher(format!(
                "Expected valid subfield matcher, got '{}'",
                data
            ))),
        }
    }

    /// Returns true, if and only if the given subfield matches against the
    /// subfield matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::{MatcherFlags, SubfieldMatcher};
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = SubfieldMatcher::new("0 == 'abc'")?;
    ///     let subfield = Subfield::new('0', "abc")?;
    ///
    ///     assert!(matcher.is_match(&subfield, &MatcherFlags::new()));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, subfield: &Subfield, flags: &MatcherFlags) -> bool {
        let case_cmp = |lhs: &BString, rhs: &BString| -> bool {
            if flags.ignore_case {
                lhs.to_lowercase() == rhs.to_lowercase()
            } else {
                lhs == rhs
            }
        };

        match self {
            Self::Comparison(codes, ComparisonOp::Eq, value) => {
                codes.contains(&subfield.code())
                    && case_cmp(subfield.value(), value)
            }
            Self::Comparison(codes, ComparisonOp::Ne, value) => {
                codes.contains(&subfield.code())
                    && !case_cmp(subfield.value(), value)
            }
            Self::Comparison(codes, ComparisonOp::StartsWith, value) => {
                codes.contains(&subfield.code())
                    && if flags.ignore_case {
                        subfield
                            .value()
                            .to_lowercase()
                            .starts_with(&value.to_lowercase())
                    } else {
                        subfield.value().starts_with(value)
                    }
            }
            Self::Comparison(codes, ComparisonOp::EndsWith, value) => {
                codes.contains(&subfield.code())
                    && if flags.ignore_case {
                        subfield
                            .value()
                            .to_lowercase()
                            .ends_with(&value.to_lowercase())
                    } else {
                        subfield.value().ends_with(value)
                    }
            }
            Self::Comparison(codes, ComparisonOp::Similar, value) => {
                if codes.contains(&subfield.code()) {
                    let flag = flags.ignore_case;
                    let lhs = maybe_lowercase!(subfield.value().to_vec(), flag);
                    let rhs = maybe_lowercase!(value.to_vec(), flag);

                    let score = normalized_levenshtein(
                        &lhs.to_str_lossy(),
                        &rhs.to_str_lossy(),
                    );

                    score > flags.strsim_threshold
                } else {
                    false
                }
            }
            Self::Comparison(_, _, _) => unreachable!(),
            Self::Regex(codes, regex, invert) => {
                let re = RegexBuilder::new(regex)
                    .case_insensitive(flags.ignore_case)
                    .build()
                    .unwrap();

                let mut result = codes.contains(&subfield.code())
                    && re.is_match(subfield.value());

                if *invert {
                    result = !result;
                }

                result
            }
            Self::In(codes, values, invert) => {
                let mut result = codes.contains(&subfield.code())
                    && values
                        .iter()
                        .any(|x: &BString| case_cmp(subfield.value(), x));

                if *invert {
                    result = !result;
                }

                result
            }
            Self::Exists(codes) => codes.contains(&subfield.code()),
        }
    }
}

fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        preceded(
            char('['),
            cut(terminated(many1(parse_subfield_code), char(']'))),
        ),
        map(char('*'), |_| {
            "0123456789abcdefghijklmnopqrstuvwxyz"
                .chars()
                .collect::<Vec<char>>()
        }),
        many1(parse_subfield_code),
        map(parse_subfield_code, |x| vec![x]),
    ))(i)
}

fn parse_subfield_matcher_comparison(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        tuple((
            ws(parse_subfield_codes),
            ws(parse_comparison_op_bstring),
            ws(parse_string),
        )),
        |(codes, op, value)| {
            SubfieldMatcher::Comparison(codes, op, BString::from(value))
        },
    )(i)
}

fn parse_subfield_matcher_regex(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        tuple((
            parse_subfield_codes,
            alt((value(false, ws(tag("=~"))), value(true, ws(tag("!~"))))),
            verify(parse_string, |x| Regex::new(x).is_ok()),
        )),
        |(codes, invert, regex)| SubfieldMatcher::Regex(codes, regex, invert),
    )(i)
}

fn parse_subfield_matcher_in(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        tuple((
            parse_subfield_codes,
            opt(ws(tag("not"))),
            ws(tag("in")),
            preceded(
                ws(char('[')),
                cut(terminated(
                    separated_list1(
                        ws(char(',')),
                        map(parse_string, BString::from),
                    ),
                    ws(char(']')),
                )),
            ),
        )),
        |(codes, invert, _, values)| {
            SubfieldMatcher::In(codes, values, invert.is_some())
        },
    )(i)
}

pub(crate) fn parse_subfield_matcher_exists(
    i: &[u8],
) -> ParseResult<SubfieldMatcher> {
    map(
        terminated(ws(parse_subfield_codes), char('?')),
        SubfieldMatcher::Exists,
    )(i)
}

pub(crate) fn parse_subfield_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    alt((
        ws(parse_subfield_matcher_comparison),
        ws(parse_subfield_matcher_regex),
        ws(parse_subfield_matcher_in),
        ws(parse_subfield_matcher_exists),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_parse_subfield_codes() -> TestResult {
        assert_eq!(parse_subfield_codes(b"[abc]")?.1, vec!['a', 'b', 'c']);
        assert_eq!(parse_subfield_codes(b"abc")?.1, vec!['a', 'b', 'c']);
        assert_eq!(parse_subfield_codes(b"a")?.1, vec!['a']);
        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher() -> TestResult {
        assert!(parse_subfield_matcher(b"[a0] == 'abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] != 'abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] =$ 'abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] =^ 'abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] =~ '^abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] !~ '^abc'").is_ok());
        assert!(parse_subfield_matcher(b"[a0] in ['a', 'b']").is_ok());
        assert!(parse_subfield_matcher(b"[a0] not in ['a', 'b']").is_ok());
        assert!(parse_subfield_matcher(b"[a0]?").is_ok());
        Ok(())
    }

    #[test]
    fn test_subfield_matcher_invalid() -> TestResult {
        assert!(SubfieldMatcher::new("[aä] == 'abc'").is_err());
        assert!(SubfieldMatcher::new("! == 'abc'").is_err());
        assert!(SubfieldMatcher::new("! == 'abc").is_err());
        Ok(())
    }

    #[test]
    fn test_subfield_matcher_eq() -> TestResult {
        let flags = MatcherFlags::new();

        let matcher = SubfieldMatcher::new("0 == 'abc'")?;
        assert!(matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("[01] == 'abc'")?;
        assert!(matcher.is_match(&Subfield::new('0', "abc")?, &flags));
        assert!(matcher.is_match(&Subfield::new('1', "abc")?, &flags));
        assert!(!matcher.is_match(&Subfield::new('2', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 == 'def'")?;
        assert!(!matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 == 'ABC'")?;
        assert!(!matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 == 'ABC'")?;
        assert!(matcher
            .is_match(&Subfield::new('0', "abc")?, &flags.ignore_case(true)));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_ne() -> TestResult {
        let flags = MatcherFlags::new();

        let matcher = SubfieldMatcher::new("0 != 'abc'")?;
        assert!(matcher.is_match(&Subfield::new('0', "def")?, &flags));
        assert!(!matcher.is_match(&Subfield::new('1', "def")?, &flags));

        let matcher = SubfieldMatcher::new("[01] != 'abc'")?;
        assert!(matcher.is_match(&Subfield::new('0', "def")?, &flags));
        assert!(matcher.is_match(&Subfield::new('1', "def")?, &flags));

        let matcher = SubfieldMatcher::new("0 != 'abc'")?;
        assert!(!matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 != 'ABC'")?;
        assert!(matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 != 'ABC'")?;
        assert!(!matcher
            .is_match(&Subfield::new('0', "abc")?, &flags.ignore_case(true)));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_starts_with() -> TestResult {
        let flags = MatcherFlags::new();

        let matcher = SubfieldMatcher::new("0 =^ 'ab'")?;
        assert!(matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("[01] =^ 'ab'")?;
        assert!(matcher.is_match(&Subfield::new('0', "abc")?, &flags));
        assert!(matcher.is_match(&Subfield::new('1', "abc")?, &flags));
        assert!(!matcher.is_match(&Subfield::new('2', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 =^ 'ab'")?;
        assert!(!matcher.is_match(&Subfield::new('0', "bcd")?, &flags));

        let matcher = SubfieldMatcher::new("0 =^ 'AB'")?;
        assert!(!matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 =^ 'AB'")?;
        assert!(matcher
            .is_match(&Subfield::new('0', "abc")?, &flags.ignore_case(true)));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_ends_with() -> TestResult {
        let flags = MatcherFlags::new();

        let matcher = SubfieldMatcher::new("0 =$ 'bc'")?;
        assert!(matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 =$ 'bc'")?;
        assert!(!matcher.is_match(&Subfield::new('0', "bcd")?, &flags));

        let matcher = SubfieldMatcher::new("0 =$ 'BC'")?;
        assert!(!matcher.is_match(&Subfield::new('0', "abc")?, &flags));

        let matcher = SubfieldMatcher::new("0 =$ 'BC'")?;
        assert!(matcher
            .is_match(&Subfield::new('0', "abc")?, &flags.ignore_case(true)));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_similar() -> TestResult {
        let matcher = SubfieldMatcher::new("0 =* 'Heike'")?;
        let flags = MatcherFlags::new();
        assert!(matcher.is_match(&Subfield::new('0', "Heike")?, &flags));

        let matcher = SubfieldMatcher::new("0 =* 'Heike'")?;
        let flags = MatcherFlags::new();
        assert!(!matcher.is_match(&Subfield::new('0', "Heiko")?, &flags));

        let matcher = SubfieldMatcher::new("0 =* 'Heike'")?;
        let flags = MatcherFlags::new().strsim_threshold(0.7);
        assert!(matcher.is_match(&Subfield::new('0', "Heiko")?, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_regex() -> TestResult {
        let matcher = SubfieldMatcher::new("0 =~ '^A.*C$'")?;
        let flags = MatcherFlags::new();
        assert!(matcher.is_match(&Subfield::new('0', "AbC")?, &flags));

        let matcher = SubfieldMatcher::new("0 =~ '^A.*C$'")?;
        let flags = MatcherFlags::new();
        assert!(!matcher.is_match(&Subfield::new('0', "abC")?, &flags));

        let matcher = SubfieldMatcher::new("0 =~ '^A.*C$'")?;
        let flags = MatcherFlags::new().ignore_case(true);
        assert!(matcher.is_match(&Subfield::new('0', "abC")?, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_in() -> TestResult {
        let matcher = SubfieldMatcher::new("0 in ['a', 'b', 'c']")?;
        let flags = MatcherFlags::new();

        assert!(matcher.is_match(&Subfield::new('0', "a")?, &flags));
        assert!(matcher.is_match(&Subfield::new('0', "b")?, &flags));
        assert!(matcher.is_match(&Subfield::new('0', "c")?, &flags));
        assert!(!matcher.is_match(&Subfield::new('0', "A")?, &flags));

        let matcher = SubfieldMatcher::new("[01] in ['a', 'b']")?;
        let flags = MatcherFlags::new();

        assert!(matcher.is_match(&Subfield::new('0', "a")?, &flags));
        assert!(matcher.is_match(&Subfield::new('0', "b")?, &flags));
        assert!(matcher.is_match(&Subfield::new('1', "a")?, &flags));
        assert!(matcher.is_match(&Subfield::new('1', "b")?, &flags));
        assert!(!matcher.is_match(&Subfield::new('2', "a")?, &flags));

        let matcher = SubfieldMatcher::new("0 not in ['a', 'b', 'c']")?;
        let flags = MatcherFlags::new();

        assert!(!matcher.is_match(&Subfield::new('0', "a")?, &flags));
        assert!(!matcher.is_match(&Subfield::new('0', "b")?, &flags));
        assert!(!matcher.is_match(&Subfield::new('0', "c")?, &flags));
        assert!(matcher.is_match(&Subfield::new('0', "A")?, &flags));

        let matcher = SubfieldMatcher::new("0 in ['a', 'b', 'c']")?;
        let flags = MatcherFlags::new().ignore_case(true);
        assert!(matcher.is_match(&Subfield::new('0', "A")?, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_exists() -> TestResult {
        let matcher = SubfieldMatcher::new("0?")?;
        let flags = MatcherFlags::new();

        assert!(matcher.is_match(&Subfield::new('0', "a")?, &flags));
        assert!(!matcher.is_match(&Subfield::new('1', "a")?, &flags));

        let matcher = SubfieldMatcher::new("[01]?")?;
        let flags = MatcherFlags::new();

        assert!(matcher.is_match(&Subfield::new('0', "a")?, &flags));
        assert!(matcher.is_match(&Subfield::new('1', "a")?, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_to_string() -> TestResult {
        let matchers = vec![
            ("0  == 'abc' ", "0 == 'abc'"),
            ("[01]  == 'abc' ", "[01] == 'abc'"),
            ("0? ", "0?"),
            ("[23]? ", "[23]?"),
            ("[2]? ", "2?"),
            ("0 in [ 'a', 'b' ]", "0 in ['a', 'b']"),
            ("[03] in [ 'a', 'b' ]", "[03] in ['a', 'b']"),
            ("0 =~ '^O[^bc]'", "0 =~ '^O[^bc]'"),
            ("[45] =~ '^O[^bc]'", "[45] =~ '^O[^bc]'"),
        ];

        for (matcher, expected) in matchers {
            assert_eq!(SubfieldMatcher::new(matcher)?.to_string(), expected);
        }

        Ok(())
    }
}
