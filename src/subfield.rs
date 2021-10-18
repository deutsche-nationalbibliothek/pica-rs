//! This module contains data structures and functions related to
//! PICA+ subfield.

use std::fmt;
use std::str::{self, FromStr};

use bstr::{BString, ByteSlice};
use regex::{Regex, RegexBuilder};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, recognize, verify};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};
use nom::Finish;

use crate::common::{
    parse_comparison_op, parse_string, ws, ComparisonOp, MatcherFlags,
    ParseResult,
};
use crate::error::{Error, Result};

/// A PICA+ subfield, that may contian invalid UTF-8 data.
#[derive(Debug, Clone, PartialEq)]
pub struct Subfield {
    pub(crate) code: char,
    pub(crate) value: BString,
}

/// Parses a subfield code.
pub(crate) fn parse_subfield_code(i: &[u8]) -> ParseResult<char> {
    map(satisfy(|c| c.is_ascii_alphanumeric()), char::from)(i)
}

/// Parses a class of subfield codes or a single subfield code.
#[inline]
fn parse_subfield_codes(i: &[u8]) -> ParseResult<Vec<char>> {
    alt((
        preceded(
            char('['),
            cut(terminated(many1(parse_subfield_code), char(']'))),
        ),
        map(parse_subfield_code, |x| vec![x]),
    ))(i)
}

/// Parses a subfield value.
fn parse_subfield_value(i: &[u8]) -> ParseResult<BString> {
    map(recognize(many0(is_not("\x1E\x1F"))), BString::from)(i)
}

/// Parses a subfield.
pub(crate) fn parse_subfield(i: &[u8]) -> ParseResult<Subfield> {
    map(
        preceded(
            char('\x1F'),
            cut(pair(parse_subfield_code, parse_subfield_value)),
        ),
        |(code, value)| Subfield::from_unchecked(code, value),
    )(i)
}

impl Subfield {
    /// Creates a new `Subfield`
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    /// use std::error::Error;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     assert!(Subfield::new('0', "12283643X").is_ok());
    ///     assert!(Subfield::new('!', "12283643X").is_err());
    ///     assert!(Subfield::new('a', "123\x1f34").is_err());
    ///     assert!(Subfield::new('a', "123\x1e34").is_err());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S>(code: char, value: S) -> Result<Subfield>
    where
        S: Into<BString>,
    {
        if !code.is_ascii_alphanumeric() {
            return Err(Error::InvalidSubfield(format!(
                "Invalid subfield code '{}'",
                code
            )));
        }

        let value: BString = value.into();
        if value.contains(&b'\x1e') || value.contains(&b'\x1f') {
            return Err(Error::InvalidSubfield(
                "Invalid subfield value.".to_string(),
            ));
        }

        Ok(Subfield { code, value })
    }

    /// Creates a new `Subfield` without checking the input
    #[inline]
    pub(crate) fn from_unchecked<S>(code: char, value: S) -> Self
    where
        S: Into<BString>,
    {
        Self {
            code,
            value: value.into(),
        }
    }

    /// Get a reference to the subfield's code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "12283643X")?;
    ///     assert_eq!(subfield.code(), '0');
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn code(&self) -> char {
        self.code
    }

    /// Get a reference to the subfield's value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "12283643X")?;
    ///     assert_eq!(subfield.value(), "12283643X");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn value(&self) -> &BString {
        &self.value
    }

    /// Returns `true` if the subfield value is valid UTF-8 byte sequence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{Error, Field, Subfield};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let subfield = Subfield::new('0', "123456789X")?;
    ///     assert_eq!(subfield.validate().is_ok(), true);
    ///
    ///     let subfield = Subfield::new('0', vec![0, 159])?;
    ///     assert_eq!(subfield.validate().is_err(), true);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.value.is_ascii() {
            return Ok(());
        }

        if let Err(e) = std::str::from_utf8(&self.value) {
            return Err(Error::Utf8Error(e));
        }

        Ok(())
    }

    /// Write the subfield into the given writer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::{PicaWriter, Subfield, WriterBuilder};
    /// use std::error::Error;
    /// use tempfile::Builder;
    /// # use std::fs::read_to_string;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn Error>> {
    ///     let mut tempfile = Builder::new().tempfile()?;
    ///     # let path = tempfile.path().to_owned();
    ///
    ///     let subfield = Subfield::new('0', "123456789X")?;
    ///     let mut writer = WriterBuilder::new().from_writer(tempfile);
    ///     subfield.write(&mut writer)?;
    ///     writer.finish()?;
    ///
    ///     # let result = read_to_string(path)?;
    ///     # assert_eq!(result, String::from("\x1f0123456789X"));
    ///     Ok(())
    /// }
    /// ```
    pub fn write(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> crate::error::Result<()> {
        write!(writer, "\x1f{}{}", self.code, self.value)?;
        Ok(())
    }
}

impl fmt::Display for Subfield {
    /// Format the subfield in a human-readable format.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}{}", self.code, self.value)
    }
}

impl Serialize for Subfield {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Subfield", 2)?;
        state.serialize_field("name", &self.code)?;
        // SAFETY: It's save because `Serialize` is only implemented for
        // `StringRecord` and not for `ByteRecord`.
        unsafe {
            state.serialize_field("value", &self.value.to_str_unchecked())?;
        }
        state.end()
    }
}

/// A `Subfield` matcher
#[derive(Debug, PartialEq)]
pub enum SubfieldMatcher {
    Comparison(Vec<char>, ComparisonOp, Vec<BString>),
    Not(Box<SubfieldMatcher>),
    Exists(Vec<char>),
}

/// parses a comparison expression
#[inline]
fn parse_subfield_matcher_comparison(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        tuple((
            ws(parse_subfield_codes),
            ws(parse_comparison_op),
            map(ws(parse_string), BString::from),
        )),
        |(codes, op, value)| {
            SubfieldMatcher::Comparison(codes, op, vec![value])
        },
    )(i)
}

/// Parses a regex comparison expression
#[inline]
fn parse_subfield_matcher_regex(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        separated_pair(
            ws(parse_subfield_codes),
            ws(tag("=~")),
            map(
                verify(ws(parse_string), |x| Regex::new(x).is_ok()),
                BString::from,
            ),
        ),
        |(codes, regex)| {
            SubfieldMatcher::Comparison(codes, ComparisonOp::Re, vec![regex])
        },
    )(i)
}

/// Parses a "in" expression
#[inline]
fn parse_subfield_matcher_in(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        tuple((
            ws(parse_subfield_codes),
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
        |(codes, not, _, values)| {
            let mut matcher =
                SubfieldMatcher::Comparison(codes, ComparisonOp::In, values);

            if not.is_some() {
                matcher = SubfieldMatcher::Not(Box::new(matcher));
            }

            matcher
        },
    )(i)
}

/// Parses a "exists" expression
#[inline]
fn parse_subfield_matcher_exists(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        terminated(ws(parse_subfield_codes), char('?')),
        SubfieldMatcher::Exists,
    )(i)
}

/// Parses a "not" expression
#[inline]
fn parse_subfield_matcher_not(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        preceded(ws(char('!')), cut(parse_subfield_matcher)),
        |matcher| SubfieldMatcher::Not(Box::new(matcher)),
    )(i)
}

/// Parses a subfield matcher expression.
#[inline]
fn parse_subfield_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    alt((
        ws(parse_subfield_matcher_comparison),
        ws(parse_subfield_matcher_regex),
        ws(parse_subfield_matcher_in),
        ws(parse_subfield_matcher_exists),
        ws(parse_subfield_matcher_not),
    ))(i)
}

impl SubfieldMatcher {
    pub fn is_match(&self, subfield: &Subfield, flags: &MatcherFlags) -> bool {
        let compare_fn = |lhs: &BString, rhs: &BString| -> bool {
            if flags.ignore_case {
                lhs.to_lowercase() == rhs.to_lowercase()
            } else {
                lhs == rhs
            }
        };

        match self {
            SubfieldMatcher::Comparison(codes, ComparisonOp::Eq, values) => {
                codes.contains(&subfield.code())
                    && compare_fn(subfield.value(), &values[0])
            }
            SubfieldMatcher::Comparison(codes, ComparisonOp::Ne, values) => {
                codes.contains(&subfield.code())
                    && !compare_fn(subfield.value(), &values[0])
            }
            SubfieldMatcher::Comparison(
                codes,
                ComparisonOp::StartsWith,
                values,
            ) => {
                codes.contains(&subfield.code())
                    && if flags.ignore_case {
                        subfield
                            .value()
                            .to_lowercase()
                            .starts_with(&values[0].to_lowercase())
                    } else {
                        subfield.value().starts_with(&values[0])
                    }
            }
            SubfieldMatcher::Comparison(
                codes,
                ComparisonOp::EndsWith,
                values,
            ) => {
                codes.contains(&subfield.code())
                    && if flags.ignore_case {
                        subfield
                            .value()
                            .to_lowercase()
                            .ends_with(&values[0].to_lowercase())
                    } else {
                        subfield.value().ends_with(&values[0])
                    }
            }
            SubfieldMatcher::Comparison(codes, ComparisonOp::Re, values) => {
                let re = RegexBuilder::new(unsafe {
                    str::from_utf8_unchecked(values[0].as_bytes())
                })
                .case_insensitive(flags.ignore_case)
                .build()
                .unwrap();

                codes.contains(&subfield.code())
                    && re.is_match(str::from_utf8(subfield.value()).unwrap())
            }
            SubfieldMatcher::Comparison(codes, ComparisonOp::In, values) => {
                codes.contains(&subfield.code())
                    && values
                        .iter()
                        .any(|x: &BString| compare_fn(subfield.value(), x))
            }
            SubfieldMatcher::Exists(codes) => codes.contains(&subfield.code()),
            SubfieldMatcher::Not(matcher) => !matcher.is_match(subfield, flags),
        }
    }
}

impl FromStr for SubfieldMatcher {
    type Err = crate::error::Error;

    /// Parse a `SubfieldMatcher` from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::SubfieldMatcher;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = SubfieldMatcher::from_str("0 == '0123456789X'");
    ///     assert!(matcher.is_ok());
    ///
    ///     Ok(())
    /// }
    /// ```
    fn from_str(s: &str) -> Result<Self> {
        match all_consuming(parse_subfield_matcher)(s.as_bytes()).finish() {
            Ok((_, tag)) => Ok(tag),
            Err(_) => Err(Error::InvalidSubfieldMatcher(
                "Invalid subfield matcher!".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::common::ComparisonOp;
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_subfield_new() -> TestResult {
        assert_eq!(
            Subfield::new('0', "abc")?,
            Subfield {
                code: '0',
                value: BString::from("abc")
            }
        );

        assert!(Subfield::new('!', "abc").is_err());
        assert!(Subfield::new('0', "a\x1fc").is_err());
        assert!(Subfield::new('0', "a\x1ec").is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_from_unchecked() -> TestResult {
        assert_eq!(
            Subfield::from_unchecked('0', "abc"),
            Subfield {
                code: '0',
                value: BString::from("abc")
            }
        );

        Ok(())
    }

    #[test]
    fn test_subfield_code() -> TestResult {
        let subfield = Subfield::new('a', "abc")?;
        assert_eq!(subfield.code(), 'a');

        Ok(())
    }

    #[test]
    fn test_subfield_value() -> TestResult {
        let subfield = Subfield::new('a', "abc")?;
        assert_eq!(subfield.value(), &BString::from("abc"));

        Ok(())
    }

    #[test]
    fn test_subfield_validate() -> TestResult {
        let subfield = Subfield::new('0', "123456789X")?;
        assert!(subfield.validate().is_ok());

        let subfield = Subfield::new('0', vec![0, 157])?;
        assert!(subfield.validate().is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_write() -> TestResult {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let subfield = Subfield::new('0', "123456789X")?;
        subfield.write(&mut writer)?;

        assert_eq!(String::from_utf8(writer.into_inner())?, "\x1f0123456789X");

        Ok(())
    }

    #[test]
    fn test_subfield_fmt() -> TestResult {
        let subfield = Subfield::new('0', "123456789X")?;
        assert_eq!(format!("{}", subfield), "$0123456789X");

        Ok(())
    }

    #[test]
    fn test_parse_subfield_code() -> TestResult {
        assert_eq!(parse_subfield_code(b"0")?.1, '0');
        assert_eq!(parse_subfield_code(b"a")?.1, 'a');
        assert_eq!(parse_subfield_code(b"Z")?.1, 'Z');
        assert!(parse_subfield_code(b"!").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_subfield_value() -> TestResult {
        assert_eq!(parse_subfield_value(b"abc")?.1, "abc");
        assert_eq!(parse_subfield_value(b"a\x1ebc")?.1, "a");
        assert_eq!(parse_subfield_value(b"a\x1fbc")?.1, "a");
        assert_eq!(parse_subfield_value(b"")?.1, "");

        Ok(())
    }

    #[test]
    fn test_parse_subfield() -> TestResult {
        assert_eq!(parse_subfield(b"\x1fa123")?.1, Subfield::new('a', "123")?);
        assert_eq!(parse_subfield(b"\x1fa")?.1, Subfield::new('a', "")?);
        assert!(parse_subfield(b"a123").is_err());
        assert!(parse_subfield(b"").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_subfield_codes() -> TestResult {
        assert_eq!(parse_subfield_codes(b"a")?.1, vec!['a']);
        assert_eq!(parse_subfield_codes(b"[abc]")?.1, vec!['a', 'b', 'c']);
        assert!(parse_subfield_codes(b"[ab").is_err());
        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_codes() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"[ab] == 'test'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a', 'b'],
                ComparisonOp::Eq,
                vec![BString::from("test")]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_ws() -> TestResult {
        assert!(parse_subfield_matcher(b" a == 'test'").is_ok());
        assert!(parse_subfield_matcher(b"a == 'test' ").is_ok());
        assert!(parse_subfield_matcher(b"a  == 'test'").is_ok());
        assert!(parse_subfield_matcher(b"a ==  'test'").is_ok());
        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_comparison_eq() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_comparison(b"a == 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::Eq,
                vec![BString::from("foobar")]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_comparison_ne() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_comparison(b"a != 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::Ne,
                vec![BString::from("foobar")]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_comparison_starts_with() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_comparison(b"a =^ 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::StartsWith,
                vec![BString::from("foobar")]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_comparison_ends_with() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_comparison(b"a =$ 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::EndsWith,
                vec![BString::from("foobar")]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_regex() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_regex(b"a =~ '^(foo|bar)$'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::Re,
                vec![BString::from("^(foo|bar)$")]
            )
        );

        assert!(parse_subfield_matcher_regex(b"0 =~ '^Tp[123]($'").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_in() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_in(b"0 in [ '123'  , '456' ] ")?.1,
            SubfieldMatcher::Comparison(
                vec!['0'],
                ComparisonOp::In,
                vec![BString::from("123"), BString::from("456")]
            )
        );

        assert_eq!(
            parse_subfield_matcher_in(b"0 not in ['123', '456']")?.1,
            SubfieldMatcher::Not(Box::new(SubfieldMatcher::Comparison(
                vec!['0'],
                ComparisonOp::In,
                vec![BString::from("123"), BString::from("456")]
            )))
        );

        assert!(parse_subfield_matcher_in(b"0 in []").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_exists() -> TestResult {
        assert_eq!(
            parse_subfield_matcher_exists(b" 0? ")?.1,
            SubfieldMatcher::Exists(vec!['0'])
        );

        assert_eq!(
            parse_subfield_matcher_exists(b"[ab]?")?.1,
            SubfieldMatcher::Exists(vec!['a', 'b'])
        );

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_from_str() -> TestResult {
        assert_eq!(
            SubfieldMatcher::from_str("0 == '0123456789X'")?,
            SubfieldMatcher::Comparison(
                vec!['0'],
                ComparisonOp::Eq,
                vec![BString::from("0123456789X")]
            )
        );

        assert!(SubfieldMatcher::from_str("foo").is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_is_match_eq() -> TestResult {
        let subfield = Subfield::new('0', "abc")?;

        let matcher = SubfieldMatcher::from_str("0 == 'abc'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 == 'aBc'")?;
        let flags = MatcherFlags { ignore_case: true };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 == 'aBc'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&subfield, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_is_match_ne() -> TestResult {
        let subfield = Subfield::new('0', "abc")?;

        let matcher = SubfieldMatcher::from_str("0 != 'def'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 != 'aBc'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 != 'aBc'")?;
        let flags = MatcherFlags { ignore_case: true };
        assert!(!matcher.is_match(&subfield, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_is_match_starts_with() -> TestResult {
        let subfield = Subfield::new('0', "abc")?;

        let matcher = SubfieldMatcher::from_str("0 =^ 'ab'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 =^ ' ab'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 =^ 'aB'")?;
        let flags = MatcherFlags { ignore_case: true };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 =^ 'aB'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&subfield, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_is_match_ends_with() -> TestResult {
        let subfield = Subfield::new('0', "abc")?;

        let matcher = SubfieldMatcher::from_str("0 =$ 'bc'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 =$ 'bc '")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 =$ 'Bc'")?;
        let flags = MatcherFlags { ignore_case: true };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 =$ 'Bc'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&subfield, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_is_match_regex() -> TestResult {
        let subfield = Subfield::new('0', "abc")?;

        let matcher = SubfieldMatcher::from_str("0 =~ '^a'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 =~ '^A'")?;
        let flags = MatcherFlags { ignore_case: true };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 =~ '^b'")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&subfield, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_is_match_in() -> TestResult {
        let subfield = Subfield::new('0', "abc")?;

        let matcher = SubfieldMatcher::from_str("0 in ['abc', 'def']")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 in ['deF', 'abC']")?;
        let flags = MatcherFlags { ignore_case: true };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("0 in ['def', 'hij']")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&subfield, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_is_match_exists() -> TestResult {
        let subfield = Subfield::new('0', "abc")?;

        let matcher = SubfieldMatcher::from_str("0?")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("[ab01]?")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(matcher.is_match(&subfield, &flags));

        let matcher = SubfieldMatcher::from_str("a?")?;
        let flags = MatcherFlags { ignore_case: false };
        assert!(!matcher.is_match(&subfield, &flags));

        Ok(())
    }
}
