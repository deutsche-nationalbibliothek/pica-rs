use std::fmt::{self, Display};
use std::ops::{BitAnd, BitOr, BitXor};

use bstr::ByteSlice;
use regex::bytes::{RegexBuilder, RegexSetBuilder};
use smallvec::SmallVec;
use strsim::normalized_levenshtein;
use winnow::Parser;

use super::parse::{
    parse_cardinality_matcher, parse_in_matcher, parse_regex_matcher,
    parse_regex_set_matcher, parse_relation_matcher,
    parse_singleton_matcher, parse_subfield_matcher,
};
use super::{
    BooleanOp, MatcherOptions, ParseMatcherError, Quantifier,
    RelationalOp,
};
use crate::matcher::parse::parse_exists_matcher;
use crate::primitives::{SubfieldCode, SubfieldRef};

#[derive(Debug, Clone, PartialEq)]
pub struct ExistsMatcher {
    pub(crate) codes: SmallVec<[SubfieldCode; 4]>,
    pub(crate) raw_data: String,
}

impl ExistsMatcher {
    /// Creates a new [ExistsMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// exists matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::ExistsMatcher;
    ///
    /// let _matcher = ExistsMatcher::new("a?")?;
    /// let _matcher = ExistsMatcher::new("[a-c]?")?;
    /// let _matcher = ExistsMatcher::new("*?")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_exists_matcher.parse(matcher.as_bytes()).map_err(|_| {
            ParseMatcherError(format!(
                "invalid exists matcher '{matcher}'"
            ))
        })
    }

    /// Checks whether list of subfields contains at least one subfield
    /// with a code of the matcher's list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{ExistsMatcher, MatcherOptions};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let matcher = ExistsMatcher::new("a?")?;
    ///
    /// let subfield = SubfieldRef::new('a', "abc")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let subfield = SubfieldRef::new('b', "def")?;
    /// assert!(!matcher.is_match(&subfield, &options));
    ///
    /// let subfields = vec![
    ///     SubfieldRef::new('b', "abc")?,
    ///     SubfieldRef::new('a', "def")?,
    /// ];
    ///
    /// assert!(matcher.is_match(&subfields, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
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

impl Display for ExistsMatcher {
    /// Format the exists matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{ExistsMatcher, MatcherOptions};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let matcher = ExistsMatcher::new("[a0-3]?")?;
    /// assert_eq!(matcher.to_string(), "[a0-3]?");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_data)
    }
}

/// A matcher that checks relations between (string) values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationMatcher {
    pub(crate) quantifier: Quantifier,
    pub(crate) codes: SmallVec<[SubfieldCode; 4]>,
    pub(crate) op: RelationalOp,
    pub(crate) value: Vec<u8>,
    pub(crate) raw_data: String,
}

impl RelationMatcher {
    /// Creates a new [RelationMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// relation matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::RelationMatcher;
    ///
    /// let _matcher = RelationMatcher::new("0 == 'Tp1'")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_relation_matcher
            .parse(matcher.as_bytes())
            .map_err(|_| {
                ParseMatcherError(format!(
                    "invalid relation matcher '{matcher}'"
                ))
            })
    }

    /// Returns true if at least one subfield is found, when the
    /// subfield's value and the matcher value are related. The two
    /// values are related iff the relation defined by the operator
    /// exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RelationMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let matcher = RelationMatcher::new("0 == 'Tp1'")?;
    /// let subfield = SubfieldRef::new('0', "Tp1")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
                Equal => self.compare(value, options),
                NotEqual => !self.compare(value, options),
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RelationMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::new().case_ignore(true);
    /// let subfield = SubfieldRef::new('a', "FOO")?;
    ///
    /// let matcher = RelationMatcher::new("a == 'foo'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = RelationMatcher::new("a != 'bar'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn compare(&self, value: &[u8], options: &MatcherOptions) -> bool {
        if value.len() != self.value.len() {
            return false;
        }

        if options.case_ignore {
            self.value.to_lowercase() == value.to_lowercase()
        } else {
            self.value == value
        }
    }

    /// Returns `true` if the given values is a prefix of the matcher's
    /// value, otherwise `false`. If the `case_ignore` flag is set,
    /// both strings will be converted to lowercase first.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RelationMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::new();
    /// let subfield = SubfieldRef::new('a', "foobar")?;
    ///
    /// let matcher = RelationMatcher::new("a =^ 'foo'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = RelationMatcher::new("a !^ 'bar'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RelationMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::new();
    /// let subfield = SubfieldRef::new('a', "foobar")?;
    ///
    /// let matcher = RelationMatcher::new("a =$ 'bar'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = RelationMatcher::new("a !$ 'foo'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RelationMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let subfield = SubfieldRef::new('a', "baz")?;
    /// let matcher = RelationMatcher::new("a =* 'bar'")?;
    ///
    /// let options = MatcherOptions::new().strsim_threshold(0.65);
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let options = MatcherOptions::new().strsim_threshold(0.75);
    /// assert!(!matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RelationMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "foobar")?;
    ///
    /// let matcher = RelationMatcher::new("a =? 'foo'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = RelationMatcher::new("a =? 'bar'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

impl Display for RelationMatcher {
    /// Format the relation matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RelationMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let matcher = RelationMatcher::new("[a0-3] == 'foo'")?;
    /// assert_eq!(matcher.to_string(), "[a0-3] == 'foo'");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_data)
    }
}

/// A matcher that checks a subfield value against a regex.
#[derive(Debug, Clone, PartialEq)]
pub struct RegexMatcher {
    pub(crate) quantifier: Quantifier,
    pub(crate) codes: SmallVec<[SubfieldCode; 4]>,
    pub(crate) regex: String,
    pub(crate) invert: bool,
    pub(crate) raw_data: String,
}

impl RegexMatcher {
    /// Creates a new [RegexMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// regex matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::RegexMatcher;
    ///
    /// let _matcher = RegexMatcher::new("0 =~ '^Tp'")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_regex_matcher.parse(matcher.as_bytes()).map_err(|_| {
            ParseMatcherError(format!(
                "invalid regex matcher '{matcher}'"
            ))
        })
    }

    /// Returns true if at least one (ANY) or all (ALL) subfield values
    /// matches against the regular expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RegexMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('0', "Tp1")?;
    ///
    /// let matcher = RegexMatcher::new("0 =~ '^Tp'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = RegexMatcher::new("0 !~ '^Ts'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let re = RegexBuilder::new(&self.regex)
            .case_insensitive(options.case_ignore)
            .build()
            .unwrap();

        let mut subfields = subfields
            .into_iter()
            .filter(|s| self.codes.contains(s.code()));

        let r#fn = |subfield: &SubfieldRef| -> bool {
            match self.invert {
                false => re.is_match(subfield.value().as_ref()),
                true => !re.is_match(subfield.value().as_ref()),
            }
        };

        match self.quantifier {
            Quantifier::All => subfields.all(r#fn),
            Quantifier::Any => subfields.any(r#fn),
        }
    }
}

impl Display for RegexMatcher {
    /// Format the regex matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RegexMatcher};
    ///
    /// let matcher = RegexMatcher::new("ALL [ab] =~ '^f.*o$'")?;
    /// assert_eq!(matcher.to_string(), "ALL [ab] =~ '^f.*o$'");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_data)
    }
}

/// A matcher that checks a subfield value against a regex set.
#[derive(Debug, Clone, PartialEq)]
pub struct RegexSetMatcher {
    pub(crate) quantifier: Quantifier,
    pub(crate) codes: SmallVec<[SubfieldCode; 4]>,
    pub(crate) regex: Vec<String>,
    pub(crate) invert: bool,
    pub(crate) raw_data: String,
}

impl RegexSetMatcher {
    /// Creates a new [RegexSetMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// regex-set matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::RegexMatcher;
    ///
    /// let _matcher = RegexMatcher::new("0 =~ '^Tp'")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_regex_set_matcher.parse(matcher.as_bytes()).map_err(
            |_| {
                ParseMatcherError(format!(
                    "invalid regex-set matcher '{matcher}'"
                ))
            },
        )
    }

    /// Returns true if at least one (ANY) or all (ALL) subfield values
    /// matches against the regular expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RegexSetMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('0', "Tp1")?;
    ///
    /// let matcher = RegexSetMatcher::new("0 =~ ['^Ts', '^Tp']")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = RegexSetMatcher::new("0 !~ ['^Ts', '^Tu']")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let re = RegexSetBuilder::new(&self.regex)
            .case_insensitive(options.case_ignore)
            .build()
            .unwrap();

        let mut subfields = subfields
            .into_iter()
            .filter(|s| self.codes.contains(s.code()));

        let r#fn = |subfield: &SubfieldRef| -> bool {
            match self.invert {
                false => re.is_match(subfield.value().as_ref()),
                true => !re.is_match(subfield.value().as_ref()),
            }
        };

        match self.quantifier {
            Quantifier::All => subfields.all(r#fn),
            Quantifier::Any => subfields.any(r#fn),
        }
    }
}

impl Display for RegexSetMatcher {
    /// Format the regex-set matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RegexSetMatcher};
    ///
    /// let matcher =
    ///     RegexSetMatcher::new("ANY [ab] !~ ['^f.*o$', 'bar']")?;
    /// assert_eq!(matcher.to_string(), "ANY [ab] !~ ['^f.*o$', 'bar']");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_data)
    }
}

/// A matcher that checks if a subfield value is in a predefined list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InMatcher {
    pub(crate) quantifier: Quantifier,
    pub(crate) codes: SmallVec<[SubfieldCode; 4]>,
    pub(crate) values: Vec<Vec<u8>>,
    pub(crate) invert: bool,
    pub(crate) raw_data: String,
}

impl InMatcher {
    /// Creates a new [InMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// in-matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::InMatcher;
    ///
    /// let _matcher = InMatcher::new("0 in ['Tp1', 'Tpz']")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_in_matcher.parse(matcher.as_bytes()).map_err(|_| {
            ParseMatcherError(format!("invalid in-matcher '{matcher}'"))
        })
    }

    /// Returns `true` if at least one subfield is found, where the
    /// value is contained in the matcher list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{InMatcher, MatcherOptions};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('0', "Tp1")?;
    ///
    /// let matcher = InMatcher::new("0 in ['Tp1', 'Tpz']")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = InMatcher::new("0 not in ['Ts1', 'Tsz']")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let mut subfields = subfields
            .into_iter()
            .filter(|s| self.codes.contains(s.code()));

        let r#fn = |subfield: &SubfieldRef| -> bool {
            let result = self.values.iter().any(|rhs| {
                if options.case_ignore {
                    subfield.value().to_lowercase()
                        == rhs.to_lowercase()
                } else {
                    subfield.value() == rhs
                }
            });

            if self.invert {
                !result
            } else {
                result
            }
        };

        match self.quantifier {
            Quantifier::All => subfields.all(r#fn),
            Quantifier::Any => subfields.any(r#fn),
        }
    }
}

impl Display for InMatcher {
    /// Format the in-matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{InMatcher, MatcherOptions};
    ///
    /// let matcher = InMatcher::new("ANY [ab] in ['foo', 'bar']")?;
    /// assert_eq!(matcher.to_string(), "ANY [ab] in ['foo', 'bar']");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_data)
    }
}

/// A matcher that checks the number of occurrences of a subfield.
#[derive(Debug, Clone, PartialEq)]
pub struct CardinalityMatcher {
    pub(crate) code: SubfieldCode,
    pub(crate) op: RelationalOp,
    pub(crate) value: usize,
    pub(crate) raw_data: String,
}

impl CardinalityMatcher {
    /// Creates a new [CardinalityMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// cardinality-matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::CardinalityMatcher;
    ///
    /// let _matcher = CardinalityMatcher::new("#a > 5")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_cardinality_matcher.parse(matcher.as_bytes()).map_err(
            |_| {
                ParseMatcherError(format!(
                    "invalid cardinality-matcher '{matcher}'"
                ))
            },
        )
    }

    /// Returns true of number of fields with a code equal to the
    /// matcher's code is `==`, `!=`, `>=`, `>`, `<=`, or `<` than the
    /// matcher's value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{CardinalityMatcher, MatcherOptions};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfields = vec![
    ///     SubfieldRef::new('a', "foo")?,
    ///     SubfieldRef::new('a', "bar")?,
    ///     SubfieldRef::new('b', "baz")?,
    /// ];
    ///
    /// let matcher = CardinalityMatcher::new("#a >= 2")?;
    /// assert!(matcher.is_match(&subfields, &options));
    ///
    /// let matcher = CardinalityMatcher::new("#b < 2")?;
    /// assert!(matcher.is_match(&subfields, &options));
    ///
    /// let matcher = CardinalityMatcher::new("#c == 0")?;
    /// assert!(matcher.is_match(&subfields, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
            RelationalOp::Equal => count == self.value,
            RelationalOp::NotEqual => count != self.value,
            RelationalOp::GreaterThanOrEqual => count >= self.value,
            RelationalOp::GreaterThan => count > self.value,
            RelationalOp::LessThanOrEqual => count <= self.value,
            RelationalOp::LessThan => count < self.value,
            _ => unreachable!(),
        }
    }
}

impl Display for CardinalityMatcher {
    /// Format the cardinality-matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{CardinalityMatcher, MatcherOptions};
    ///
    /// let matcher = CardinalityMatcher::new("#a >= 3")?;
    /// assert_eq!(matcher.to_string(), "#a >= 3");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_data)
    }
}

/// A matcher that checks for the singleton matcher.
///
/// This matcher combines all atomic, singleton matcher into a new
/// matcher.
#[derive(Debug, Clone, PartialEq)]
pub enum SingletonMatcher {
    Cardinality(CardinalityMatcher),
    Regex(RegexMatcher),
    RegexSet(RegexSetMatcher),
    Relation(RelationMatcher),
    Exists(ExistsMatcher),
    In(InMatcher),
}

impl SingletonMatcher {
    /// Creates a new [SingletonMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// cardinality-matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::CardinalityMatcher;
    ///
    /// let _matcher = CardinalityMatcher::new("#a > 5")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_singleton_matcher.parse(matcher.as_bytes()).map_err(
            |_| {
                ParseMatcherError(format!(
                    "invalid singleton-matcher '{matcher}'"
                ))
            },
        )
    }

    /// Returns `true` if the underlying matcher returns `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, SingletonMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "foo")?;
    ///
    /// let matcher = SingletonMatcher::new("a == 'foo'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = SingletonMatcher::new("#a == 1")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Cardinality(m) => m.is_match(subfields, options),
            Self::Regex(m) => m.is_match(subfields, options),
            Self::RegexSet(m) => m.is_match(subfields, options),
            Self::Relation(m) => m.is_match(subfields, options),
            Self::Exists(m) => m.is_match(subfields, options),
            Self::In(m) => m.is_match(subfields, options),
        }
    }
}

impl Display for SingletonMatcher {
    /// Format the singleton-matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, SingletonMatcher};
    ///
    /// let matcher = SingletonMatcher::new("#a >= 3")?;
    /// assert_eq!(matcher.to_string(), "#a >= 3");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cardinality(matcher) => write!(f, "{}", matcher),
            Self::Exists(matcher) => write!(f, "{}", matcher),
            Self::In(matcher) => write!(f, "{}", matcher),
            Self::Regex(matcher) => write!(f, "{}", matcher),
            Self::RegexSet(matcher) => write!(f, "{}", matcher),
            Self::Relation(matcher) => write!(f, "{}", matcher),
        }
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
    /// Creates a new [SubfieldMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// subfield-matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::SubfieldMatcher;
    ///
    /// let _matcher = SubfieldMatcher::new("a == 'foo'")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_subfield_matcher
            .parse(matcher.as_bytes())
            .map_err(|_| {
                ParseMatcherError(format!(
                    "invalid subfield matcher '{matcher}'"
                ))
            })
    }

    /// Returns `true` if the underlying matcher returns `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, SubfieldMatcher};
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "foo")?;
    ///
    /// let matcher = SubfieldMatcher::new("a =^ 'f' && (a =$ 'o')")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    pub fn is_match<'a>(
        &self,
        subfields: impl IntoIterator<Item = &'a SubfieldRef<'a>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        use BooleanOp::*;

        match self {
            Self::Singleton(m) => m.is_match(subfields, options),
            Self::Group(m) => m.is_match(subfields, options),
            Self::Not(m) => !m.is_match(subfields, options),
            Self::Composite { lhs, op, rhs } => {
                let lhs = lhs.is_match(subfields.clone(), options);
                match *op {
                    And => lhs && rhs.is_match(subfields, options),
                    Xor => lhs != rhs.is_match(subfields, options),
                    Or => lhs || rhs.is_match(subfields, options),
                }
            }
        }
    }
}

impl Display for SubfieldMatcher {
    /// Format the subfield-matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, SubfieldMatcher};
    ///
    /// let matcher = SubfieldMatcher::new("#a >= 3")?;
    /// assert_eq!(matcher.to_string(), "#a >= 3");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Singleton(m) => write!(f, "{}", m),
            Self::Group(m) => write!(f, "({})", m),
            Self::Not(m) => write!(f, "!{}", m),
            Self::Composite { lhs, op, rhs } => {
                write!(f, "{} {} {}", lhs, op, rhs)
            }
        }
    }
}

impl BitAnd for SubfieldMatcher {
    type Output = Self;

    #[inline]
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

    #[inline]
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

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Xor,
            rhs: Box::new(rhs),
        }
    }
}
