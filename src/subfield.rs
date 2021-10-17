//! This module contains data structures and functions related to
//! PICA+ subfield.

use std::fmt;
use std::str::FromStr;

use bstr::{BString, ByteSlice};
use regex::Regex;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, satisfy};
use nom::combinator::{all_consuming, cut, map, opt, recognize, verify};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};
use nom::Finish;

use crate::common::{
    parse_boolean_op, parse_comparison_op, parse_string, ws, BooleanOp,
    ComparisonOp, ParseResult,
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

#[derive(Debug, PartialEq)]
pub enum SubfieldMatcher {
    Comparison(Vec<char>, ComparisonOp, Vec<BString>),
    Composite(Box<SubfieldMatcher>, BooleanOp, Box<SubfieldMatcher>),
    Group(Box<SubfieldMatcher>),
    Not(Box<SubfieldMatcher>),
    Exists(Vec<char>),
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

#[inline]
fn parse_subfield_matcher_exists(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        terminated(ws(parse_subfield_codes), char('?')),
        SubfieldMatcher::Exists,
    )(i)
}

#[inline]
fn parse_subfield_matcher_group(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(parse_subfield_matcher, ws(char(')')))),
        ),
        |matcher| SubfieldMatcher::Group(Box::new(matcher)),
    )(i)
}

#[inline]
fn parse_subfield_matcher_not(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    map(
        preceded(ws(char('!')), cut(parse_subfield_matcher)),
        |matcher| SubfieldMatcher::Not(Box::new(matcher)),
    )(i)
}

#[inline]
fn parse_subfield_matcher_primary(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    alt((
        ws(parse_subfield_matcher_comparison),
        ws(parse_subfield_matcher_regex),
        ws(parse_subfield_matcher_in),
        ws(parse_subfield_matcher_exists),
        ws(parse_subfield_matcher_group),
        ws(parse_subfield_matcher_not),
    ))(i)
}

#[inline]
fn parse_subfield_matcher(i: &[u8]) -> ParseResult<SubfieldMatcher> {
    let (i, (first, remainder)) = tuple((
        parse_subfield_matcher_primary,
        many0(pair(
            ws(parse_boolean_op),
            ws(parse_subfield_matcher_primary),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, (op, next)| {
            SubfieldMatcher::Composite(Box::new(prev), op, Box::new(next))
        }),
    ))
}

#[derive(Debug)]
pub struct MatcherFlags {
    _ignore_case: bool,
}

impl SubfieldMatcher {
    pub fn is_match(&self, _subfield: &Subfield, _flags: MatcherFlags) -> bool {
        unimplemented!()
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
                vec!["test".to_string()]
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
    fn test_subfield_matcher_from_str() -> TestResult {
        assert_eq!(
            SubfieldMatcher::from_str("0 == '0123456789X'")?,
            SubfieldMatcher::Comparison(
                vec!['0'],
                ComparisonOp::Eq,
                vec!["0123456789X".to_string()]
            )
        );

        assert!(SubfieldMatcher::from_str("foo").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_eq() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"a == 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::Eq,
                vec!["foobar".to_string()]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_strict_eq() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"a === 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::StrictEq,
                vec!["foobar".to_string()]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_ne() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"a != 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::Ne,
                vec!["foobar".to_string()]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_starts_with() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"a =^ 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::StartsWith,
                vec!["foobar".to_string()]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_ends_with() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"a =$ 'foobar'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::EndsWith,
                vec!["foobar".to_string()]
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_regex() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"a =~ '^(foo|bar)$'")?.1,
            SubfieldMatcher::Comparison(
                vec!['a'],
                ComparisonOp::Re,
                vec!["^(foo|bar)$".to_string()]
            )
        );

        assert!(parse_subfield_matcher(b"0 =~ '^Tp[123]($'").is_err());

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_in() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"0 in [ '123'  , '456' ] ")?.1,
            SubfieldMatcher::Comparison(
                vec!['0'],
                ComparisonOp::In,
                vec!["123".to_string(), "456".to_string()]
            )
        );

        assert_eq!(
            parse_subfield_matcher(b"0 not in ['123', '456']")?.1,
            SubfieldMatcher::Not(Box::new(SubfieldMatcher::Comparison(
                vec!['0'],
                ComparisonOp::In,
                vec!["123".to_string(), "456".to_string()]
            )))
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_exists() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b" 0? ")?.1,
            SubfieldMatcher::Exists(vec!['0'])
        );

        assert_eq!(
            parse_subfield_matcher(b"[ab]?")?.1,
            SubfieldMatcher::Exists(vec!['a', 'b'])
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_group() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"(0?) ")?.1,
            SubfieldMatcher::Group(Box::new(SubfieldMatcher::Exists(vec![
                '0'
            ])))
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_not() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b" !0? ")?.1,
            SubfieldMatcher::Not(Box::new(SubfieldMatcher::Exists(vec!['0'])))
        );

        Ok(())
    }

    #[test]
    fn test_parse_subfield_matcher_composite() -> TestResult {
        assert_eq!(
            parse_subfield_matcher(b"0? && a? && b?")?.1,
            SubfieldMatcher::Composite(
                Box::new(SubfieldMatcher::Composite(
                    Box::new(SubfieldMatcher::Exists(vec!['0'])),
                    BooleanOp::And,
                    Box::new(SubfieldMatcher::Exists(vec!['a']))
                )),
                BooleanOp::And,
                Box::new(SubfieldMatcher::Exists(vec!['b']))
            )
        );

        assert_eq!(
            parse_subfield_matcher(b"0? || a? || b?")?.1,
            SubfieldMatcher::Composite(
                Box::new(SubfieldMatcher::Composite(
                    Box::new(SubfieldMatcher::Exists(vec!['0'])),
                    BooleanOp::Or,
                    Box::new(SubfieldMatcher::Exists(vec!['a']))
                )),
                BooleanOp::Or,
                Box::new(SubfieldMatcher::Exists(vec!['b']))
            )
        );

        assert_eq!(
            parse_subfield_matcher(b"0? || a? && b?")?.1,
            SubfieldMatcher::Composite(
                Box::new(SubfieldMatcher::Composite(
                    Box::new(SubfieldMatcher::Exists(vec!['0'])),
                    BooleanOp::Or,
                    Box::new(SubfieldMatcher::Exists(vec!['a']))
                )),
                BooleanOp::And,
                Box::new(SubfieldMatcher::Exists(vec!['b']))
            )
        );

        Ok(())
    }
}
