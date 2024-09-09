//! Matcher that works on PICA+ [Subfields](pica_record_v1::Subfield).

use std::cell::RefCell;
use std::ops::{BitAnd, BitOr, BitXor};
use std::str::FromStr;

use bstr::ByteSlice;
use pica_record_v1::parser::parse_subfield_code;
use pica_record_v1::{SubfieldCode, SubfieldRef};
use regex::bytes::{Regex, RegexBuilder};
use strsim::normalized_levenshtein;
use winnow::ascii::digit1;
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, separated, separated_pair,
    terminated,
};
use winnow::error::ParserError;
use winnow::{PResult, Parser};

use crate::common::{
    parse_quantifier, parse_relational_op_str,
    parse_relational_op_usize, parse_string, ws, BooleanOp, Quantifier,
    RelationalOp,
};
use crate::{MatcherOptions, ParseMatcherError};

/// A matcher that checks if a subfield exists.
///
/// This matcher can be used to determine if a single subfield or a
/// list of subfields contains at least one subfield with a code, that
/// is contained in the matcher's code list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistsMatcher {
    codes: Vec<SubfieldCode>,
}

const SUBFIELD_CODES: &str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[inline]
fn parse_subfield_code_range(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    separated_pair(parse_subfield_code, '-', parse_subfield_code)
        .verify(|(min, max)| min < max)
        .map(|(min, max)| {
            (min.as_byte()..=max.as_byte())
                .map(SubfieldCode::from_unchecked)
                .collect()
        })
        .parse_next(i)
}

#[inline]
fn parse_subfield_code_single(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    parse_subfield_code.map(|code| vec![code]).parse_next(i)
}

#[inline]
fn parse_subfield_code_list(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    delimited(
        '[',
        repeat(
            1..,
            alt((
                parse_subfield_code_range,
                parse_subfield_code_single,
            )),
        )
        .fold(Vec::new, |mut acc: Vec<_>, item| {
            acc.extend_from_slice(&item);
            acc
        }),
        ']',
    )
    .parse_next(i)
}

#[inline]
fn parse_subfield_code_wildcard(
    i: &mut &[u8],
) -> PResult<Vec<SubfieldCode>> {
    '*'.value(
        SUBFIELD_CODES
            .chars()
            .map(|code| SubfieldCode::new(code).unwrap())
            .collect(),
    )
    .parse_next(i)
}

/// Parse a list of subfield codes
fn parse_subfield_codes(i: &mut &[u8]) -> PResult<Vec<SubfieldCode>> {
    alt((
        parse_subfield_code_list,
        parse_subfield_code_single,
        parse_subfield_code_wildcard,
    ))
    .parse_next(i)
}

/// Parse the matcher expression from a byte slice.
pub(crate) fn parse_exists_matcher(
    i: &mut &[u8],
) -> PResult<ExistsMatcher> {
    terminated(parse_subfield_codes, '?')
        .map(|codes| ExistsMatcher { codes })
        .parse_next(i)
}

impl ExistsMatcher {
    /// Create a new exists matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::ExistsMatcher;
    /// use pica_record_v1::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = ExistsMatcher::new(vec!['0']);
    ///     let options = Default::default();
    ///
    ///     assert!(matcher
    ///         .is_match(&SubfieldRef::new('0', "123456789X"), &options));
    ///
    ///     assert!(
    ///         !matcher.is_match(&SubfieldRef::new('a', "abc"), &options)
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: Into<Vec<char>>>(codes: T) -> Self {
        let codes = codes
            .into()
            .into_iter()
            .map(|code| SubfieldCode::new(code).unwrap())
            .collect();

        Self { codes }
    }

    /// Returns `true` if at least one subfield is found with a code
    /// which is in the matcher's code list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use pica_matcher::subfield_matcher::ExistsMatcher;
    /// use pica_record_v1::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = ExistsMatcher::from_str("[103]?")?;
    ///     let options = Default::default();
    ///     assert!(
    ///         matcher.is_match(&SubfieldRef::new('0', "123"), &options)
    ///     );
    ///
    ///     let matcher = ExistsMatcher::from_str("*?")?;
    ///     let options = Default::default();
    ///     assert!(
    ///         matcher.is_match(&SubfieldRef::new('a', "abc"), &options)
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        _options: &MatcherOptions,
    ) -> bool {
        subfields
            .into_iter()
            .any(|subfield| self.codes.contains(subfield.code()))
    }
}

impl TryFrom<&[u8]> for ExistsMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_exists_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

impl FromStr for ExistsMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A matcher that checks relations between (string) values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationMatcher {
    quantifier: Quantifier,
    codes: Vec<SubfieldCode>,
    op: RelationalOp,
    value: Vec<u8>,
}

impl RelationMatcher {
    /// Create a new relation matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::RelationMatcher;
    /// use pica_record_v1::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = RelationMatcher::new("0 == '123456789X'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher
    ///         .is_match(&SubfieldRef::new('0', "123456789X"), &options));
    ///
    ///     assert!(!matcher
    ///         .is_match(&SubfieldRef::new('0', "123456789!"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(value: T) -> Self {
        Self::try_from(value.as_ref()).expect("relation matcher")
    }

    /// Returns true if at least one subfield is found, when the
    /// subfield's value and the matcher value are related. The two
    /// values are related iff the relation defined by the operator
    /// exists.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        use RelationalOp::*;

        let mut subfields = subfields
            .into_iter()
            .filter(|s| self.codes.contains(s.code()));

        let check = |subfield: &SubfieldRef| -> bool {
            let value = subfield.value().as_ref();
            match self.op {
                Eq => self.compare(value, options),
                Ne => !self.compare(value, options),
                StartsWith => self.starts_with(value, options, false),
                StartsNotWith => self.starts_with(value, options, true),
                EndsWith => self.ends_with(value, options, false),
                EndsNotWith => self.ends_with(value, options, true),
                Similar => self.is_similar(value, options),
                Contains => self.contains(value, options),
                _ => unreachable!(),
            }
        };

        match self.quantifier {
            Quantifier::All => subfields.all(check),
            Quantifier::Any => subfields.any(check),
        }
    }

    /// Returns `true` if the given value is equal to the matcher's
    /// value. If the `case_ignore` flag is set, both strings will be
    /// converted to lowercase first.
    fn compare(&self, value: &[u8], options: &MatcherOptions) -> bool {
        if options.case_ignore {
            self.value.to_lowercase() == value.to_lowercase()
        } else {
            self.value == value
        }
    }

    /// Returns `true` if the given values is a prefix of the matcher's
    /// value, otherwise `false`. If the `case_ignore` flag is set,
    /// both strings will be converted to lowercase first.
    fn starts_with(
        &self,
        value: &[u8],
        options: &MatcherOptions,
        invert: bool,
    ) -> bool {
        let mut result = if options.case_ignore {
            value.to_lowercase().starts_with(&self.value.to_lowercase())
        } else {
            value.starts_with(&self.value)
        };

        if invert {
            result = !result
        }

        result
    }

    /// Returns `true` if the given values is a suffix of the matcher's
    /// value, otherwise `false`. If the `case_ignore` flag is set,
    /// both strings will be converted to lowercase first.
    fn ends_with(
        &self,
        value: &[u8],
        options: &MatcherOptions,
        invert: bool,
    ) -> bool {
        let mut result = if options.case_ignore {
            value.to_lowercase().ends_with(&self.value.to_lowercase())
        } else {
            value.ends_with(&self.value)
        };

        if invert {
            result = !result;
        }

        result
    }

    /// Returns `true` if the given value is similar to the matcher's
    /// value. The similarity score is determined by calculating the
    /// normalized levenshtein distance between both strings. If the
    /// `case_ignore` flag is set, both strings will be converted to
    /// lowercase first.
    fn is_similar(&self, rhs: &[u8], options: &MatcherOptions) -> bool {
        let lhs = self.value.to_str_lossy();
        let rhs = rhs.to_str_lossy();

        let score = if options.case_ignore {
            normalized_levenshtein(
                &lhs.to_lowercase(),
                &rhs.to_lowercase(),
            )
        } else {
            normalized_levenshtein(&lhs, &rhs)
        };

        score > options.strsim_threshold
    }

    /// Returns `true` if the given value is a substring of the value.
    /// If the `case_ignore` flag is set, both strings will be
    /// converted to lowercase first.
    fn contains(&self, value: &[u8], options: &MatcherOptions) -> bool {
        if options.case_ignore {
            value
                .to_lowercase()
                .find(self.value.to_lowercase())
                .is_some()
        } else {
            value.find(&self.value).is_some()
        }
    }
}

/// Parse a relational expression
#[inline]
fn parse_relation_matcher(i: &mut &[u8]) -> PResult<RelationMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        ws(parse_subfield_codes),
        ws(parse_relational_op_str),
        ws(parse_string),
    )
        .map(|(quantifier, codes, op, value)| RelationMatcher {
            quantifier,
            codes,
            op,
            value,
        })
        .parse_next(i)
}

impl TryFrom<&[u8]> for RelationMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_relation_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

impl FromStr for RelationMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A matcher that checks a subfield value against a regex.
#[derive(PartialEq, Clone, Debug)]
pub struct RegexMatcher {
    quantifier: Quantifier,
    codes: Vec<SubfieldCode>,
    re: String,
    invert: bool,
}

impl RegexMatcher {
    /// Create a new regex matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::RegexMatcher;
    /// use pica_matcher::Quantifier;
    /// use pica_record_v1::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let options = Default::default();
    ///
    ///     let subfield = SubfieldRef::new('0', "Oa");
    ///     let matcher =
    ///         RegexMatcher::new(vec!['0'], "^Oa", Quantifier::Any, false);
    ///     assert!(matcher.is_match(&subfield, &options));
    ///
    ///     let subfield = SubfieldRef::new('0', "Ob");
    ///     let matcher =
    ///         RegexMatcher::new(vec!['0'], "^Oa", Quantifier::Any, true);
    ///     assert!(matcher.is_match(&subfield, &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<S, T>(
        codes: T,
        re: S,
        quantifier: Quantifier,
        invert: bool,
    ) -> Self
    where
        S: Into<String>,
        T: Into<Vec<char>>,
    {
        let codes = codes
            .into()
            .into_iter()
            .map(|code| SubfieldCode::new(code).unwrap())
            .collect();

        let re = re.into();
        assert!(RegexBuilder::new(&re).build().is_ok());

        RegexMatcher {
            quantifier,
            codes,
            re,
            invert,
        }
    }

    /// Returns true if at least one subfield value is found, that
    /// matches against the regular expression.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let re = RegexBuilder::new(&self.re)
            .case_insensitive(options.case_ignore)
            .build()
            .unwrap();

        let mut subfields = subfields
            .into_iter()
            .filter(|s| self.codes.contains(s.code()));

        let check_fn = |subfield: &SubfieldRef| -> bool {
            let mut result = re.is_match(subfield.value().as_ref());
            if self.invert {
                result = !result;
            }

            result
        };

        match self.quantifier {
            Quantifier::All => subfields.all(check_fn),
            Quantifier::Any => subfields.any(check_fn),
        }
    }
}

/// Parse a regex matcher expression
fn parse_regex_matcher(i: &mut &[u8]) -> PResult<RegexMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        ws(parse_subfield_codes),
        ws(alt(("=~".value(false), "!~".value(true)))),
        parse_string
            .verify_map(|re| String::from_utf8(re).ok())
            .verify(|re| Regex::new(re).is_ok()),
    )
        .map(|(quantifier, codes, invert, re)| RegexMatcher {
            quantifier,
            codes,
            invert,
            re,
        })
        .parse_next(i)
}

impl TryFrom<&[u8]> for RegexMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_regex_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

impl FromStr for RegexMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A matcher that checks if a subfield value is in a predefined list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InMatcher {
    quantifier: Quantifier,
    codes: Vec<SubfieldCode>,
    values: Vec<Vec<u8>>,
    invert: bool,
}

impl InMatcher {
    /// Create a new matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::InMatcher;
    /// use pica_matcher::Quantifier;
    /// use pica_record_v1::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = InMatcher::new(
    ///         vec!['0'],
    ///         vec!["abc", "def"],
    ///         Quantifier::Any,
    ///         false,
    ///     );
    ///     let options = Default::default();
    ///     assert!(
    ///         matcher.is_match(&SubfieldRef::new('0', "def"), &options)
    ///     );
    ///
    ///     let matcher = InMatcher::new(
    ///         vec!['0'],
    ///         vec!["abc", "def"],
    ///         Quantifier::Any,
    ///         true,
    ///     );
    ///     let options = Default::default();
    ///     assert!(
    ///         matcher.is_match(&SubfieldRef::new('0', "hij"), &options)
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T, U, V>(
        codes: T,
        values: U,
        quantifier: Quantifier,
        invert: bool,
    ) -> Self
    where
        T: Into<Vec<char>>,
        U: Into<Vec<V>>,
        V: AsRef<[u8]>,
    {
        let codes = codes.into();
        let values = values
            .into()
            .into_iter()
            .map(|s| s.as_ref().to_vec())
            .collect::<Vec<_>>();

        let codes = codes
            .into_iter()
            .map(|code| SubfieldCode::new(code).unwrap())
            .collect();

        Self {
            quantifier,
            codes,
            values,
            invert,
        }
    }

    /// Returns `true` if at least one subfield is found, where the
    /// value is contained in the matcher list.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let mut subfields = subfields
            .into_iter()
            .filter(|s| self.codes.contains(s.code()));

        let check_fn = |subfield: &SubfieldRef| -> bool {
            let mut result = self.values.iter().any(|rhs| {
                if options.case_ignore {
                    subfield.value().to_lowercase()
                        == rhs.to_lowercase()
                } else {
                    subfield.value() == rhs
                }
            });

            if self.invert {
                result = !result;
            }

            result
        };

        match self.quantifier {
            Quantifier::All => subfields.all(check_fn),
            Quantifier::Any => subfields.any(check_fn),
        }
    }
}

/// Parse a in matcher expression.
fn parse_in_matcher(i: &mut &[u8]) -> PResult<InMatcher> {
    (
        opt(ws(parse_quantifier)).map(Option::unwrap_or_default),
        ws(parse_subfield_codes),
        opt(ws("not")).map(|x| x.is_some()),
        preceded(
            ws("in"),
            delimited(
                ws('['),
                separated(1.., parse_string, ws(',')),
                ws(']'),
            ),
        ),
    )
        .map(|(quantifier, codes, invert, values)| InMatcher {
            quantifier,
            codes,
            invert,
            values,
        })
        .parse_next(i)
}

impl TryFrom<&[u8]> for InMatcher {
    type Error = ParseMatcherError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_in_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

impl FromStr for InMatcher {
    type Err = ParseMatcherError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A matcher that checks the number of occurrences of a subfield.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardinalityMatcher {
    code: SubfieldCode,
    op: RelationalOp,
    value: usize,
}

impl CardinalityMatcher {
    /// Create a new matcher.
    ///
    /// # Panics
    ///
    /// This function panics on ∀ invalid input. The cardinality
    /// matcher uses only a subset of all relational operators; the
    /// caller must ensure that the operator is applicable on
    /// `usize`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::CardinalityMatcher;
    /// use pica_matcher::RelationalOp;
    /// use pica_record_v1::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = CardinalityMatcher::new('0', RelationalOp::Gt, 1);
    ///     let options = Default::default();
    ///
    ///     assert!(matcher.is_match(
    ///         vec![
    ///             &SubfieldRef::new('0', "def")?,
    ///             &SubfieldRef::new('0', "abc")?
    ///         ],
    ///         &options
    ///     ));
    ///
    ///     assert!(
    ///         !matcher.is_match(&SubfieldRef::new('0', "def"), &options)
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T>(code: T, op: RelationalOp, value: usize) -> Self
    where
        T: Into<char>,
    {
        let code = code.into();

        assert!(code.is_ascii_alphanumeric());
        assert!(op.is_usize_applicable());

        Self {
            code: SubfieldCode::new(code).unwrap(),
            op,
            value,
        }
    }

    /// Returns true of number of fields with a code equal to the
    /// matcher's code is `==`, `!=`, `>=`, `>`, `<=`, or `<` than the
    /// matcher's value.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        _options: &MatcherOptions,
    ) -> bool {
        let count = subfields
            .into_iter()
            .filter(|&s| self.code == *s.code())
            .count();

        match self.op {
            RelationalOp::Eq => count == self.value,
            RelationalOp::Ne => count != self.value,
            RelationalOp::Ge => count >= self.value,
            RelationalOp::Gt => count > self.value,
            RelationalOp::Le => count <= self.value,
            RelationalOp::Lt => count < self.value,
            _ => unreachable!(),
        }
    }
}

/// Parse a cardinality matcher expression.
fn parse_cardinality_matcher(
    i: &mut &[u8],
) -> PResult<CardinalityMatcher> {
    preceded(
        ws('#'),
        (
            ws(parse_subfield_code),
            ws(parse_relational_op_usize),
            digit1
                .verify_map(|value| std::str::from_utf8(value).ok())
                .verify_map(|value| value.parse::<usize>().ok()),
        ),
    )
    .map(|(code, op, value)| CardinalityMatcher { code, op, value })
    .parse_next(i)
}

impl TryFrom<&[u8]> for CardinalityMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_cardinality_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

impl FromStr for CardinalityMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

/// A matcher that checks for the singleton matcher.
///
/// This matcher combines all atomic, singleton matcher into a new
/// matcher.
#[derive(Clone, Debug, PartialEq)]
pub enum SingletonMatcher {
    Cardinality(CardinalityMatcher),
    Exists(ExistsMatcher),
    In(InMatcher),
    Regex(RegexMatcher),
    Relation(RelationMatcher),
}

/// Parse a singleton matcher expression.
fn parse_singleton_matcher(i: &mut &[u8]) -> PResult<SingletonMatcher> {
    alt((
        parse_cardinality_matcher.map(SingletonMatcher::Cardinality),
        parse_exists_matcher.map(SingletonMatcher::Exists),
        parse_in_matcher.map(SingletonMatcher::In),
        parse_regex_matcher.map(SingletonMatcher::Regex),
        parse_relation_matcher.map(SingletonMatcher::Relation),
    ))
    .parse_next(i)
}

impl SingletonMatcher {
    /// Create a new singleton matcher from a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::SingletonMatcher;
    /// use pica_record_v1::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher = SingletonMatcher::new("0 != '123456789X'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher
    ///         .is_match(&SubfieldRef::new('0', "2345678901"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
        Self::try_from(data.as_ref()).expect("singleton matcher")
    }

    /// Returns `true` if the underlying matcher returns `true`.
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Cardinality(m) => m.is_match(subfields, options),
            Self::Exists(m) => m.is_match(subfields, options),
            Self::In(m) => m.is_match(subfields, options),
            Self::Regex(m) => m.is_match(subfields, options),
            Self::Relation(m) => m.is_match(subfields, options),
        }
    }
}

impl TryFrom<&[u8]> for SingletonMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_singleton_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

/// A matcher that allows grouping, negation and connecting of
/// singleton matcher.
#[derive(Clone, Debug, PartialEq)]
pub enum SubfieldMatcher {
    Singleton(SingletonMatcher),
    Group(Box<SubfieldMatcher>),
    Not(Box<SubfieldMatcher>),
    Composite {
        lhs: Box<SubfieldMatcher>,
        op: BooleanOp,
        rhs: Box<SubfieldMatcher>,
    },
}

impl SubfieldMatcher {
    /// Create a new matcher from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_matcher::subfield_matcher::SubfieldMatcher;
    /// use pica_record_v1::SubfieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let matcher =
    ///         SubfieldMatcher::new("0 != '123456789X' && 0 =^ '234'");
    ///     let options = Default::default();
    ///
    ///     assert!(matcher
    ///         .is_match(&SubfieldRef::new('0', "2345678901"), &options));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
        Self::try_from(data.as_ref()).expect("subfield matcher")
    }

    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Singleton(m) => m.is_match(subfields, options),
            Self::Group(m) => m.is_match(subfields, options),
            Self::Not(m) => !m.is_match(subfields, options),
            Self::Composite { lhs, op, rhs } => match op {
                BooleanOp::And => {
                    lhs.is_match(subfields.clone(), options)
                        && rhs.is_match(subfields, options)
                }
                BooleanOp::Or => {
                    lhs.is_match(subfields.clone(), options)
                        || rhs.is_match(subfields, options)
                }
                BooleanOp::Xor => {
                    lhs.is_match(subfields.clone(), options)
                        != rhs.is_match(subfields, options)
                }
            },
        }
    }
}

#[inline]
fn parse_subfield_exists_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
    parse_exists_matcher
        .map(SingletonMatcher::Exists)
        .map(SubfieldMatcher::Singleton)
        .parse_next(i)
}

#[inline]
pub(crate) fn parse_subfield_singleton_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
    parse_singleton_matcher
        .map(SubfieldMatcher::Singleton)
        .parse_next(i)
}

#[inline]
fn parse_not_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    preceded(
        ws('!'),
        alt((
            parse_group_matcher,
            parse_subfield_exists_matcher,
            parse_not_matcher,
        )),
    )
    .map(|matcher| SubfieldMatcher::Not(Box::new(matcher)))
    .parse_next(i)
}

thread_local! {
    pub static GROUP_LEVEL: RefCell<u32> = const { RefCell::new(0) };
}

fn increment_group_level(i: &mut &[u8]) -> PResult<()> {
    GROUP_LEVEL.with(|level| {
        *level.borrow_mut() += 1;
        if *level.borrow() >= 32 {
            Err(winnow::error::ErrMode::from_error_kind(
                i,
                winnow::error::ErrorKind::Many,
            ))
        } else {
            Ok(())
        }
    })
}

fn decrement_group_level() {
    GROUP_LEVEL.with(|level| {
        *level.borrow_mut() -= 1;
    })
}

#[inline]
fn parse_group_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    delimited(
        terminated(ws('('), increment_group_level),
        alt((
            parse_composite_matcher,
            parse_subfield_singleton_matcher,
            parse_not_matcher,
            parse_group_matcher,
        )),
        ws(')').map(|_| decrement_group_level()),
    )
    .map(|matcher| SubfieldMatcher::Group(Box::new(matcher)))
    .parse_next(i)
}

#[inline]
fn parse_or_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    (
        alt((
            ws(parse_group_matcher),
            ws(parse_xor_matcher),
            ws(parse_and_matcher),
            ws(parse_subfield_singleton_matcher),
            ws(parse_not_matcher),
        )),
        repeat(
            1..,
            preceded(
                ws("||"),
                alt((
                    ws(parse_group_matcher),
                    ws(parse_xor_matcher),
                    ws(parse_and_matcher),
                    ws(parse_subfield_singleton_matcher),
                    ws(parse_not_matcher),
                )),
            ),
        ),
    )
        .map(|(head, remainder): (_, Vec<_>)| {
            remainder.into_iter().fold(head, |prev, next| prev | next)
        })
        .parse_next(i)
}

#[inline]
fn parse_and_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    (
        ws(alt((
            parse_group_matcher,
            parse_singleton_matcher.map(SubfieldMatcher::Singleton),
            parse_not_matcher,
        ))),
        repeat(
            1..,
            preceded(
                ws("&&"),
                ws(alt((
                    parse_group_matcher,
                    parse_singleton_matcher
                        .map(SubfieldMatcher::Singleton),
                    parse_not_matcher,
                ))),
            ),
        ),
    )
        .map(|(head, remainder): (_, Vec<_>)| {
            remainder.into_iter().fold(head, |prev, next| prev & next)
        })
        .parse_next(i)
}

#[inline]
fn parse_xor_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    (
        ws(alt((
            parse_group_matcher,
            parse_and_matcher,
            parse_singleton_matcher.map(SubfieldMatcher::Singleton),
            parse_not_matcher,
        ))),
        repeat(
            1..,
            preceded(
                ws(alt(("^", "XOR"))),
                ws(alt((
                    parse_group_matcher,
                    parse_and_matcher,
                    parse_singleton_matcher
                        .map(SubfieldMatcher::Singleton),
                    parse_not_matcher,
                ))),
            ),
        ),
    )
        .map(|(head, remainder): (_, Vec<_>)| {
            remainder.into_iter().fold(head, |prev, next| prev ^ next)
        })
        .parse_next(i)
}

#[inline]
fn parse_composite_matcher(i: &mut &[u8]) -> PResult<SubfieldMatcher> {
    alt((parse_or_matcher, parse_xor_matcher, parse_and_matcher))
        .parse_next(i)
}

pub fn parse_subfield_matcher(
    i: &mut &[u8],
) -> PResult<SubfieldMatcher> {
    alt((
        parse_composite_matcher,
        parse_group_matcher,
        parse_not_matcher,
        parse_singleton_matcher.map(SubfieldMatcher::Singleton),
    ))
    .parse_next(i)
}

impl TryFrom<&[u8]> for SubfieldMatcher {
    type Error = ParseMatcherError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_subfield_matcher.parse(value).map_err(|_| {
            let value = value.to_str_lossy().to_string();
            ParseMatcherError::InvalidSubfieldMatcher(value)
        })
    }
}

impl FromStr for SubfieldMatcher {
    type Err = ParseMatcherError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.as_bytes())
    }
}

impl BitAnd for SubfieldMatcher {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::And,
            rhs: Box::new(rhs),
        }
    }
}

impl BitOr for SubfieldMatcher {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Or,
            rhs: Box::new(rhs),
        }
    }
}

impl BitXor for SubfieldMatcher {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Xor,
            rhs: Box::new(rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn parse_subfield_codes() {
        let codes = SUBFIELD_CODES.chars().collect::<Vec<char>>();

        macro_rules! parse_success {
            ($input:expr, $expected:expr) => {
                assert_eq!(
                    super::parse_subfield_codes.parse($input).unwrap(),
                    $expected
                );
            };
        }

        for code in codes.iter() {
            parse_success!(code.to_string().as_bytes(), vec![*code]);
        }

        parse_success!(b"*", codes);
        parse_success!(b"[12]", vec!['1', '2']);
        parse_success!(b"[1-3]", vec!['1', '2', '3']);
        parse_success!(
            b"[1-3a-cx]",
            vec!['1', '2', '3', 'a', 'b', 'c', 'x']
        );

        assert!(super::parse_subfield_codes.parse(b"!").is_err());
        assert!(super::parse_subfield_codes.parse(b"12").is_err());
        assert!(super::parse_subfield_codes.parse(b"[a1!]").is_err());
        assert!(super::parse_subfield_codes.parse(b"[2-2]").is_err());
    }

    #[test]
    fn parse_exists_matcher() -> TestResult {
        macro_rules! parse_success {
            ($input:expr, $codes:expr) => {
                assert_eq!(
                    super::parse_exists_matcher.parse($input).unwrap(),
                    ExistsMatcher { codes: $codes }
                );
            };
        }

        parse_success!(
            b"*?",
            SUBFIELD_CODES
                .chars()
                .map(|code| SubfieldCode::new(code).unwrap())
                .collect()
        );
        parse_success!(
            b"[a-f]?",
            vec![
                'a'.try_into()?,
                'b'.try_into()?,
                'c'.try_into()?,
                'd'.try_into()?,
                'e'.try_into()?,
                'f'.try_into()?
            ]
        );
        parse_success!(
            b"[a-cf]?",
            vec![
                'a'.try_into()?,
                'b'.try_into()?,
                'c'.try_into()?,
                'f'.try_into()?
            ]
        );
        parse_success!(
            b"[ab]?",
            vec!['a'.try_into()?, 'b'.try_into()?]
        );
        parse_success!(b"a?", vec!['a'.try_into()?]);

        assert!(super::parse_exists_matcher.parse(b"a ?").is_err());

        Ok(())
    }

    #[test]
    fn parse_relation_matcher() -> TestResult {
        use Quantifier::*;
        use RelationalOp::*;

        use super::parse_relation_matcher;

        macro_rules! parse_success {
            ($input:expr, $quantifier:expr, $codes:expr, $op:expr, $value:expr) => {
                assert_eq!(
                    parse_relation_matcher.parse($input).unwrap(),
                    RelationMatcher {
                        quantifier: $quantifier,
                        codes: $codes,
                        op: $op,
                        value: $value.to_vec()
                    }
                );
            };
        }

        parse_success!(
            b"0 == 'abc'",
            Any,
            vec!['0'.try_into()?],
            Eq,
            b"abc"
        );
        parse_success!(
            b"0 != 'abc'",
            Any,
            vec!['0'.try_into()?],
            Ne,
            b"abc"
        );
        parse_success!(
            b"0 =^ 'abc'",
            Any,
            vec!['0'.try_into()?],
            StartsWith,
            b"abc"
        );
        parse_success!(
            b"0 !^ 'abc'",
            Any,
            vec!['0'.try_into()?],
            StartsNotWith,
            b"abc"
        );
        parse_success!(
            b"0 =$ 'abc'",
            Any,
            vec!['0'.try_into()?],
            EndsWith,
            b"abc"
        );
        parse_success!(
            b"0 !$ 'abc'",
            Any,
            vec!['0'.try_into()?],
            EndsNotWith,
            b"abc"
        );
        parse_success!(
            b"0 =* 'abc'",
            Any,
            vec!['0'.try_into()?],
            Similar,
            b"abc"
        );
        parse_success!(
            b"0 =? 'abc'",
            Any,
            vec!['0'.try_into()?],
            Contains,
            b"abc"
        );

        assert!(parse_relation_matcher.parse(b"0 >= 'abc'").is_err());
        assert!(parse_relation_matcher.parse(b"0 > 'abc'").is_err());
        assert!(parse_relation_matcher.parse(b"0 <= 'abc'").is_err());
        assert!(parse_relation_matcher.parse(b"0 < 'abc'").is_err());

        Ok(())
    }

    #[test]
    fn parse_regex_matcher() -> TestResult {
        use super::parse_regex_matcher;

        macro_rules! parse_success {
            ($input:expr, $codes:expr, $re:expr, $invert:expr) => {
                assert_eq!(
                    parse_regex_matcher.parse($input).unwrap(),
                    RegexMatcher {
                        quantifier: Quantifier::Any,
                        codes: $codes,
                        invert: $invert,
                        re: $re.to_string()
                    }
                );
            };
        }

        parse_success!(
            b"0 =~ '^Tp'",
            vec!['0'.try_into()?],
            "^Tp",
            false
        );
        parse_success!(
            b"0 !~ '^Tp'",
            vec!['0'.try_into()?],
            "^Tp",
            true
        );
        parse_success!(
            b"[ab] =~ 'foo'",
            vec!['a'.try_into()?, 'b'.try_into()?],
            "foo",
            false
        );

        assert!(parse_regex_matcher.parse(b"0 =~ '[[ab]'").is_err());
        assert!(parse_regex_matcher.parse(b"0 !~ '[[ab]'").is_err());

        Ok(())
    }
}
