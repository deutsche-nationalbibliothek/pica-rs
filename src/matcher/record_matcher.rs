use std::fmt;
use std::ops::{BitAnd, BitOr, Not};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{all_consuming, cut, map, map_res, opt};
use nom::multi::many1;
use nom::sequence::{preceded, terminated, tuple};
use nom::Finish;

use crate::common::{ws, ParseResult};
use crate::matcher::*;
use crate::{ByteRecord, Error};

use super::subfield_matcher::parse_subfield_matcher_exists;

#[derive(Debug, PartialEq)]
pub enum RecordMatcher {
    Singleton(Box<FieldMatcher>),
    Group(Box<RecordMatcher>),
    Not(Box<RecordMatcher>),
    Composite(Box<RecordMatcher>, BooleanOp, Box<RecordMatcher>),
    Cardinality(
        TagMatcher,
        OccurrenceMatcher,
        Option<Box<SubfieldListMatcher>>,
        ComparisonOp,
        usize,
    ),
    True,
}

impl fmt::Display for RecordMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Singleton(expr) => expr.fmt(f),
            Self::Group(expr) => write!(f, "({})", expr),
            Self::Not(expr) => write!(f, "!{}", expr),
            Self::Composite(lhs, op, rhs) => {
                write!(f, "{} {} {}", lhs, op, rhs)
            }
            Self::Cardinality(tm, om, sm, op, value) => {
                if let Some(sm) = sm {
                    write!(f, "#{}{}{{{}}} {} {}", tm, om, sm, op, value)
                } else {
                    write!(f, "#{}{} {} {}", tm, om, op, value)
                }
            }
            Self::True => write!(f, "True"),
        }
    }
}

impl RecordMatcher {
    /// Creates a record matcher from a string slice.
    ///
    /// If an invalid record matcher is given, an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::RecordMatcher;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     assert!(RecordMatcher::new("013A? && 012A/*{0? && 0 == 'abc'}").is_ok());
    ///     assert!(RecordMatcher::new("013!?").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S: AsRef<str>>(data: S) -> Result<Self, Error> {
        let data = data.as_ref();

        match all_consuming(parse_record_matcher)(data.as_bytes()).finish() {
            Ok((_, matcher)) => Ok(matcher),
            Err(_) => Err(Error::InvalidMatcher(format!(
                "Expected valid record matcher, got '{}'",
                data
            ))),
        }
    }

    /// Returns true, if and only if the given record matches against
    /// the record matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica::matcher::{MatcherFlags, RecordMatcher};
    /// use pica::ByteRecord;
    /// use std::str::FromStr;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let matcher = RecordMatcher::new("012A/*{0? && 0 == 'abc'}")?;
    ///     let record = ByteRecord::from_bytes("012A/01 \x1f0abc\x1e")?;
    ///     assert!(matcher.is_match(&record, &MatcherFlags::default()));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match(&self, record: &ByteRecord, flags: &MatcherFlags) -> bool {
        match self {
            Self::Singleton(matcher) => {
                record.iter().any(|field| matcher.is_match(field, flags))
            }
            Self::Group(matcher) => matcher.is_match(record, flags),
            Self::Not(matcher) => !matcher.is_match(record, flags),
            Self::Composite(lhs, BooleanOp::And, rhs) => {
                lhs.is_match(record, flags) && rhs.is_match(record, flags)
            }
            Self::Composite(lhs, BooleanOp::Or, rhs) => {
                lhs.is_match(record, flags) || rhs.is_match(record, flags)
            }
            Self::Cardinality(tag, occurrence, subfields, op, value) => {
                let fields = record
                    .iter()
                    .filter(|field| {
                        tag.is_match(field.tag())
                            && occurrence.is_match(field.occurrence())
                    })
                    .filter(|field| {
                        if let Some(matcher) = subfields {
                            matcher.is_match(field.subfields(), flags)
                        } else {
                            true
                        }
                    });

                let cardinality = fields.count();

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
            Self::True => true,
        }
    }
}

impl From<FieldMatcher> for RecordMatcher {
    fn from(matcher: FieldMatcher) -> Self {
        RecordMatcher::Singleton(Box::new(matcher))
    }
}

impl BitAnd for RecordMatcher {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        RecordMatcher::Composite(Box::new(self), BooleanOp::And, Box::new(rhs))
    }
}

impl BitOr for RecordMatcher {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        RecordMatcher::Composite(Box::new(self), BooleanOp::Or, Box::new(rhs))
    }
}

impl Not for RecordMatcher {
    type Output = Self;

    fn not(self) -> Self::Output {
        RecordMatcher::Not(Box::new(self))
    }
}

fn parse_record_matcher_singleton(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(ws(parse_field_matcher), |x| {
        RecordMatcher::Singleton(Box::new(x))
    })(i)
}

fn parse_record_matcher_exists(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(
        alt((
            ws(parse_field_matcher_exists),
            map(
                tuple((
                    parse_tag_matcher,
                    parse_occurrence_matcher,
                    preceded(char('.'), cut(parse_subfield_matcher_exists)),
                )),
                |(tag, occurrence, subfields)| {
                    FieldMatcher::Subield(
                        tag,
                        occurrence,
                        SubfieldListMatcher::Singleton(subfields),
                    )
                },
            ),
        )),
        |x| RecordMatcher::Singleton(Box::new(x)),
    )(i)
}

fn parse_record_matcher_group(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(
        preceded(
            ws(char('(')),
            cut(terminated(
                alt((
                    parse_record_matcher_composite,
                    parse_record_matcher_singleton,
                    parse_record_matcher_not,
                    parse_record_matcher_cardinality,
                    parse_record_matcher_group,
                )),
                ws(char(')')),
            )),
        ),
        |matcher| RecordMatcher::Group(Box::new(matcher)),
    )(i)
}

fn parse_record_matcher_not(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(
        preceded(
            ws(char('!')),
            cut(alt((
                parse_record_matcher_group,
                parse_record_matcher_exists,
                parse_record_matcher_not,
            ))),
        ),
        |matcher| RecordMatcher::Not(Box::new(matcher)),
    )(i)
}

fn parse_record_matcher_composite_and(i: &[u8]) -> ParseResult<RecordMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_record_matcher_group),
            ws(parse_record_matcher_cardinality),
            ws(parse_record_matcher_singleton),
            ws(parse_record_matcher_not),
            ws(parse_record_matcher_exists),
        )),
        many1(preceded(
            ws(tag("&&")),
            alt((
                ws(parse_record_matcher_group),
                ws(parse_record_matcher_cardinality),
                ws(parse_record_matcher_singleton),
                ws(parse_record_matcher_not),
                ws(parse_record_matcher_exists),
            )),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev & next),
    ))
}

fn parse_record_matcher_composite_or(i: &[u8]) -> ParseResult<RecordMatcher> {
    let (i, (first, remainder)) = tuple((
        alt((
            ws(parse_record_matcher_group),
            ws(parse_record_matcher_composite_and),
            ws(parse_record_matcher_cardinality),
            ws(parse_record_matcher_singleton),
            ws(parse_record_matcher_not),
        )),
        many1(preceded(
            ws(tag("||")),
            cut(alt((
                ws(parse_record_matcher_group),
                ws(parse_record_matcher_composite_and),
                ws(parse_record_matcher_cardinality),
                ws(parse_record_matcher_singleton),
                ws(parse_record_matcher_not),
            ))),
        )),
    ))(i)?;

    Ok((
        i,
        remainder.into_iter().fold(first, |prev, next| prev | next),
    ))
}

fn parse_record_matcher_composite(i: &[u8]) -> ParseResult<RecordMatcher> {
    alt((
        parse_record_matcher_composite_or,
        parse_record_matcher_composite_and,
    ))(i)
}

fn parse_record_matcher_cardinality(i: &[u8]) -> ParseResult<RecordMatcher> {
    map(
        preceded(
            ws(char('#')),
            cut(tuple((
                ws(parse_tag_matcher),
                ws(parse_occurrence_matcher),
                opt(preceded(
                    ws(char('{')),
                    cut(terminated(parse_subfield_list_matcher, ws(char('}')))),
                )),
                ws(parse_comparison_op_usize),
                map_res(digit1, |s| {
                    std::str::from_utf8(s).unwrap().parse::<usize>()
                }),
            ))),
        ),
        |(t, o, s, op, value)| {
            RecordMatcher::Cardinality(t, o, s.map(Box::new), op, value)
        },
    )(i)
}

pub(crate) fn parse_record_matcher(i: &[u8]) -> ParseResult<RecordMatcher> {
    alt((
        ws(parse_record_matcher_composite),
        ws(parse_record_matcher_group),
        ws(parse_record_matcher_not),
        ws(parse_record_matcher_singleton),
        ws(parse_record_matcher_cardinality),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestResult;

    #[test]
    fn test_record_matcher_invalid() -> TestResult {
        assert!(RecordMatcher::new("003@ .0 == '123456789X'").is_err());
        Ok(())
    }

    #[test]
    fn test_record_matcher_singleton() -> TestResult {
        let matcher = RecordMatcher::new("003@.0 == '123456789X'")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        let matcher = RecordMatcher::new("003@.0 == '123456789X'")?;
        let record = ByteRecord::from_bytes("003@ \x1f023456789X1\x1e")?;
        assert!(!matcher.is_match(&record, &MatcherFlags::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_group() -> TestResult {
        // composite
        let matcher =
            RecordMatcher::new("(#003@ == 1 && 003@.0 == '123456789X')")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        // singleton
        let matcher = RecordMatcher::new("(003@.0 == '123456789X')")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        // not
        let matcher = RecordMatcher::new("(!012A?)")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        // group
        let matcher = RecordMatcher::new("(((003@.0 == '123456789X')))")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_not() -> TestResult {
        // group
        let matcher = RecordMatcher::new("!(003@.0 == '123456789X')")?;
        let record = ByteRecord::from_bytes("003@ \x1f0223456789X1\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        // exists
        let matcher = RecordMatcher::new("!012A?")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        let matcher = RecordMatcher::new("!012A.0?")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        // not
        let matcher = RecordMatcher::new("!!003@?")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_composite() -> TestResult {
        let matcher = RecordMatcher::new("003@? && 003@.0 == '123456789X'")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));

        let matcher = RecordMatcher::new("!012A? && 003@.0 == '123456789X'")?;
        let record = ByteRecord::from_bytes("003@ \x1f0123456789X\x1e")?;
        assert!(matcher.is_match(&record, &MatcherFlags::default()));
        Ok(())
    }

    #[test]
    fn test_record_matcher_to_string() -> TestResult {
        let values = vec![
            ("003@.0  ==  '0123456789'", "003@.0 == '0123456789'"),
            ("( 003@.0  ==  '0123456789')", "(003@.0 == '0123456789')"),
            ("!012A.0?", "!012A.0?"),
            ("!012A.0? && 013A.a == 'abc'", "!012A.0? && 013A.a == 'abc'"),
            ("!012A.0? || 013A.a == 'abc'", "!012A.0? || 013A.a == 'abc'"),
            ("#012A{ a? && b == '1'} >= 2", "#012A{a? && b == '1'} >= 2"),
            ("#012A >= 2", "#012A >= 2"),
        ];

        for (matcher, expected) in values {
            assert_eq!(RecordMatcher::new(matcher)?.to_string(), expected);
        }

        Ok(())
    }
}
