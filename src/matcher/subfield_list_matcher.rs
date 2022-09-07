use std::fmt;
use std::ops::{BitAnd, BitOr};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{all_consuming, cut, map, map_res};
use nom::multi::many1;
use nom::sequence::{preceded, terminated, tuple};
use nom::Finish;

use super::subfield_matcher::parse_subfield_matcher_exists;
use crate::common::{ws, ParseResult};
use crate::matcher::{
    parse_comparison_op_usize, parse_subfield_matcher, BooleanOp,
    ComparisonOp, MatcherFlags, SubfieldMatcher,
};
use crate::subfield::{parse_subfield_code, Subfield};
use crate::Error;

#[derive(Debug, PartialEq)]
pub enum SubfieldListMatcher {
    Singleton(SubfieldMatcher),
    Group(Box<SubfieldListMatcher>),
    Not(Box<SubfieldListMatcher>),
    Composite(
        Box<SubfieldListMatcher>,
        BooleanOp,
        Box<SubfieldListMatcher>,
    ),
    Cardinality(char, ComparisonOp, usize),
}

impl fmt::Display for SubfieldListMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Singleton(expr) => write!(f, "{}", expr),
            Self::Group(expr) => write!(f, "({})", expr),
            Self::Not(expr) => write!(f, "!{}", expr),
            Self::Composite(lhs, op, rhs) => {
                write!(f, "{} {} {}", lhs, op, rhs)
            }
            Self::Cardinality(code, op, value) => {
                write!(f, "#{} {} {}", code, op, value)
            }
        }
    }
}

impl SubfieldListMatcher {
    /// Creates a subfield list matcher from a string slice.
    ///
    /// If an invalid subfield list matcher is given, an error is
    /// returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::SubfieldListMatcher;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(SubfieldListMatcher::new("0 == 'abc' && 9?").is_ok());
    ///     assert!(SubfieldListMatcher::new("0 == 'ab' && !?").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        let data = data.as_ref();

        match all_consuming(parse_subfield_list_matcher)(
            data.as_bytes(),
        )
        .finish()
        {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(Error::InvalidMatcher(format!(
                "Expected valid subfield list matcher, got '{}'",
                data
            ))),
        }
    }

    /// Returns true, if and only if the given subfield list matches
    /// against the subfield list matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::{MatcherFlags, SubfieldListMatcher};
    /// use pica::Subfield;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = SubfieldListMatcher::new("0 == 'abc' && 9?")?;
    ///     let list =
    ///         [Subfield::new('0', "abc")?, Subfield::new('9', "123")?];
    ///     assert!(matcher.is_match(&list, &MatcherFlags::default()));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(
        &self,
        subfields: &[Subfield],
        flags: &MatcherFlags,
    ) -> bool {
        match self {
            Self::Singleton(matcher) => {
                subfields.iter().any(|s| matcher.is_match(s, flags))
            }
            Self::Group(matcher) => matcher.is_match(subfields, flags),
            Self::Not(matcher) => !matcher.is_match(subfields, flags),
            Self::Composite(lhs, BooleanOp::And, rhs) => {
                lhs.is_match(subfields, flags)
                    && rhs.is_match(subfields, flags)
            }
            Self::Composite(lhs, BooleanOp::Or, rhs) => {
                lhs.is_match(subfields, flags)
                    || rhs.is_match(subfields, flags)
            }
            Self::Cardinality(code, op, value) => {
                let cardinality = subfields
                    .iter()
                    .filter(|s| s.code() == *code)
                    .count();

                match op {
                    ComparisonOp::Eq => cardinality == *value,
                    ComparisonOp::Ne => cardinality != *value,
                    ComparisonOp::Gt => cardinality > *value,
                    ComparisonOp::Ge => cardinality >= *value,
                    ComparisonOp::Lt => cardinality < *value,
                    ComparisonOp::Le => cardinality <= *value,
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl BitAnd for SubfieldListMatcher {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        SubfieldListMatcher::Composite(
            Box::new(self),
            BooleanOp::And,
            Box::new(rhs),
        )
    }
}

impl BitOr for SubfieldListMatcher {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        SubfieldListMatcher::Composite(
            Box::new(self),
            BooleanOp::Or,
            Box::new(rhs),
        )
    }
}

impl From<SubfieldMatcher> for SubfieldListMatcher {
    fn from(matcher: SubfieldMatcher) -> Self {
        SubfieldListMatcher::Singleton(matcher)
    }
}

pub(crate) fn parse_subfield_list_matcher_singleton(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    map(ws(parse_subfield_matcher), SubfieldListMatcher::Singleton)(i)
}

pub(crate) fn parse_subfield_list_matcher_exists(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    map(
        ws(parse_subfield_matcher_exists),
        SubfieldListMatcher::Singleton,
    )(i)
}

fn parse_subfield_list_matcher_cardinality(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
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
        |(code, op, value)| {
            SubfieldListMatcher::Cardinality(code, op, value)
        },
    )(i)
}

fn parse_subfield_list_matcher_group(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(
                alt((
                    parse_subfield_list_matcher_composite,
                    parse_subfield_list_matcher_singleton,
                    parse_subfield_list_matcher_not,
                    parse_subfield_list_matcher_group,
                )),
                ws(char(')')),
            )),
        ),
        |matcher| SubfieldListMatcher::Group(Box::new(matcher)),
    )(i)
}

fn parse_subfield_list_matcher_not(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    map(
        preceded(
            ws(char('!')),
            cut(alt((
                parse_subfield_list_matcher_group,
                parse_subfield_list_matcher_exists,
                parse_subfield_list_matcher_not,
            ))),
        ),
        |matcher| SubfieldListMatcher::Not(Box::new(matcher)),
    )(i)
}

fn parse_subfield_list_matcher_composite_and(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_subfield_list_matcher_group),
            ws(parse_subfield_list_matcher_singleton),
            ws(parse_subfield_list_matcher_cardinality),
            ws(parse_subfield_list_matcher_not),
        )),
        many1(preceded(
            ws(tag("&&")),
            alt((
                ws(parse_subfield_list_matcher_group),
                ws(parse_subfield_list_matcher_singleton),
                ws(parse_subfield_list_matcher_cardinality),
                ws(parse_subfield_list_matcher_not),
            )),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev & next),
    ))
}

fn parse_subfield_list_matcher_composite_or(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_subfield_list_matcher_group),
            ws(parse_subfield_list_matcher_composite_and),
            ws(parse_subfield_list_matcher_singleton),
            ws(parse_subfield_list_matcher_cardinality),
            ws(parse_subfield_list_matcher_not),
        )),
        many1(preceded(
            ws(tag("||")),
            cut(alt((
                ws(parse_subfield_list_matcher_group),
                ws(parse_subfield_list_matcher_composite_and),
                ws(parse_subfield_list_matcher_singleton),
                ws(parse_subfield_list_matcher_cardinality),
                ws(parse_subfield_list_matcher_not),
            ))),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev | next),
    ))
}

fn parse_subfield_list_matcher_composite(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    alt((
        parse_subfield_list_matcher_composite_or,
        parse_subfield_list_matcher_composite_and,
    ))(i)
}

pub(crate) fn parse_subfield_list_matcher(
    i: &[u8],
) -> ParseResult<SubfieldListMatcher> {
    alt((
        parse_subfield_list_matcher_composite,
        parse_subfield_list_matcher_group,
        parse_subfield_list_matcher_not,
        parse_subfield_list_matcher_singleton,
        parse_subfield_list_matcher_cardinality,
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_subfield_list_matcher_singleton() -> TestResult {
        let matcher = SubfieldListMatcher::new("0 == 'abc'")?;

        let subfields = [Subfield::new('0', "abc")?];
        assert!(matcher.is_match(&subfields, &MatcherFlags::default()));

        let subfields = [Subfield::new('0', "bcd")?];
        assert!(!matcher.is_match(&subfields, &MatcherFlags::default()));

        Ok(())
    }

    #[test]
    fn test_subfield_list_matcher_cardinality() -> TestResult {
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];

        let matcher = SubfieldListMatcher::new("#0 == 2")?;
        assert!(matcher.is_match(&subfields, &MatcherFlags::default()));

        let matcher = SubfieldListMatcher::new("#0 >= 2")?;
        assert!(matcher.is_match(&subfields, &MatcherFlags::default()));

        let matcher = SubfieldListMatcher::new("#0 <= 2")?;
        assert!(matcher.is_match(&subfields, &MatcherFlags::default()));

        let matcher = SubfieldListMatcher::new("#0 != 2")?;
        assert!(!matcher.is_match(&subfields, &MatcherFlags::default()));

        let matcher = SubfieldListMatcher::new("#0 < 2")?;
        assert!(!matcher.is_match(&subfields, &MatcherFlags::default()));

        let matcher = SubfieldListMatcher::new("#0 > 2")?;
        assert!(!matcher.is_match(&subfields, &MatcherFlags::default()));

        assert!(SubfieldListMatcher::new("#0 == 'abc'").is_err());
        assert!(SubfieldListMatcher::new("#0 == -1").is_err());
        assert!(SubfieldListMatcher::new("##0 == 1").is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_list_matcher_group() -> TestResult {
        let flags = MatcherFlags::default();

        let matcher = SubfieldListMatcher::new("(0? && 0 == 'abc')")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher = SubfieldListMatcher::new("(0 == 'abc')")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher = SubfieldListMatcher::new("(!9?)")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher = SubfieldListMatcher::new("((0 == 'def'))")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        assert!(SubfieldListMatcher::new("((0 == 'abc')").is_err());

        Ok(())
    }

    #[test]
    fn test_subfield_list_matcher_not() -> TestResult {
        let flags = MatcherFlags::default();

        let matcher = SubfieldListMatcher::new("!(0? && 0 == 'hij')")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher = SubfieldListMatcher::new("!9?")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher = SubfieldListMatcher::new("!!(0? && 0 == 'abc')")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_list_matcher_composite() -> TestResult {
        let flags = MatcherFlags::default();

        let matcher =
            SubfieldListMatcher::new("0? && 0 == 'abc' && 0 == 'def'")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher =
            SubfieldListMatcher::new("0 == 'abc' && 0 == 'def'")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "hij")?];
        assert!(!matcher.is_match(&subfields, &flags));

        let matcher =
            SubfieldListMatcher::new("0 == 'abc' && 0 == 'def'")?;
        let subfields =
            [Subfield::new('0', "hij")?, Subfield::new('0', "def")?];
        assert!(!matcher.is_match(&subfields, &flags));

        let matcher =
            SubfieldListMatcher::new("0 == 'abc' || 0 == 'def'")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "hij")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher =
            SubfieldListMatcher::new("0 == 'abc' || 0 == 'def'")?;
        let subfields =
            [Subfield::new('0', "hij")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher =
            SubfieldListMatcher::new("9? || 0 == 'abc' && 0 == 'def'")?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher = SubfieldListMatcher::new(
            "9? && 0 == 'abc' ||  0 == 'def'",
        )?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(matcher.is_match(&subfields, &flags));

        let matcher = SubfieldListMatcher::new(
            "9? && 0 == 'abc' ||  0 == 'hij'",
        )?;
        let subfields =
            [Subfield::new('0', "abc")?, Subfield::new('0', "def")?];
        assert!(!matcher.is_match(&subfields, &flags));

        Ok(())
    }

    #[test]
    fn test_subfield_list_matcher_to_string() -> TestResult {
        let values = vec![
            ("0 == 'abc'", "0 == 'abc'"),
            ("( a == 'abc' )", "(a == 'abc')"),
            ("!( a == 'abc' )", "!(a == 'abc')"),
            (
                "a == 'a'  && b == 'b'  && c == 'c' ",
                "a == 'a' && b == 'b' && c == 'c'",
            ),
            (
                "a == 'a'  || b == 'b'  || c == 'c' ",
                "a == 'a' || b == 'b' || c == 'c'",
            ),
            ("#a  >=  3", "#a >= 3"),
        ];

        for (matcher, expected) in values {
            assert_eq!(
                SubfieldListMatcher::new(matcher)?.to_string(),
                expected
            );
        }

        Ok(())
    }
}
