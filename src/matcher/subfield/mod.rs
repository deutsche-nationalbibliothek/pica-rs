//! Matcher that can be applied on a list of [SubfieldRef].

use std::fmt::{self, Display};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not,
};

use bstr::ByteSlice;
use parser::{
    parse_cardinality_matcher, parse_exists_matcher, parse_in_matcher,
    parse_regex_matcher, parse_regex_set_matcher,
    parse_relation_matcher, parse_singleton_matcher,
    parse_subfield_matcher,
};
use regex::bytes::{RegexBuilder, RegexSetBuilder};
use smallvec::SmallVec;
use strsim::normalized_levenshtein;
use winnow::Parser;

use super::{
    BooleanOp, MatcherOptions, ParseMatcherError, Quantifier,
    RelationalOp,
};
use crate::primitives::{SubfieldCode, SubfieldRef};

/// A matcher that checks for the existance of subfields.
#[derive(Debug, Clone, PartialEq)]
pub struct ExistsMatcher {
    pub(crate) codes: SmallVec<[SubfieldCode; 4]>,
    pub(crate) raw_data: String,
}

pub(crate) mod parser;

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
    /// use pica_record::matcher::subfield::ExistsMatcher;
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
    /// use pica_record::matcher::subfield::ExistsMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::ExistsMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RelationMatcher;
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
    /// use pica_record::matcher::subfield::RelationMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::subfield::RelationMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RelationMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RelationMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RelationMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RelationMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RelationMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RegexMatcher;
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
    /// use pica_record::matcher::subfield::RegexMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RegexMatcher;
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
    /// use pica_record::matcher::subfield::RegexMatcher;
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
    /// use pica_record::matcher::subfield::RegexSetMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::RegexSetMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::InMatcher;
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
    /// use pica_record::matcher::subfield::InMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::InMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::CardinalityMatcher;
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
    /// use pica_record::matcher::subfield::CardinalityMatcher;
    /// use pica_record::matcher::MatcherOptions;
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

impl Display for CardinalityMatcher {
    /// Format the cardinality-matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::subfield::CardinalityMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::CardinalityMatcher;
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
    /// use pica_record::matcher::subfield::SingletonMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
    /// use pica_record::matcher::subfield::SingletonMatcher;
    ///
    /// let matcher = SingletonMatcher::new("#a >= 3")?;
    /// assert_eq!(matcher.to_string(), "#a >= 3");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cardinality(m) => write!(f, "{m}"),
            Self::Exists(m) => write!(f, "{m}"),
            Self::In(m) => write!(f, "{m}"),
            Self::Regex(m) => write!(f, "{m}"),
            Self::RegexSet(m) => write!(f, "{m}"),
            Self::Relation(m) => write!(f, "{m}"),
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
    /// use pica_record::matcher::subfield::SubfieldMatcher;
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
    /// use pica_record::matcher::subfield::SubfieldMatcher;
    /// use pica_record::matcher::MatcherOptions;
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
                    Xor => lhs != rhs.is_match(subfields, options),
                    And => lhs && rhs.is_match(subfields, options),
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
    /// use pica_record::matcher::subfield::SubfieldMatcher;
    ///
    /// let matcher = SubfieldMatcher::new("#a >= 3")?;
    /// assert_eq!(matcher.to_string(), "#a >= 3");
    ///
    /// let mut matcher = SubfieldMatcher::new("a == 'foo' || a == 'bar'")?;
    /// matcher &= SubfieldMatcher::new("c == 'baz'")?;
    /// assert_eq!(
    ///     matcher.to_string(),
    ///     "(a == 'foo' || a == 'bar') && c == 'baz'"
    /// );
    ///
    /// let mut matcher = SubfieldMatcher::new("a == 'foo'")?;
    /// matcher &= SubfieldMatcher::new("b == 'bar' ^ b == 'baz'")?;
    /// assert_eq!(
    ///     matcher.to_string(),
    ///     "a == 'foo' && (b == 'bar' ^ b == 'baz')"
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Singleton(m) => write!(f, "{m}"),
            Self::Group(m) => write!(f, "({m})"),
            Self::Not(m) => write!(f, "!{m}"),
            Self::Composite { lhs, op, rhs } => {
                write!(f, "{lhs} {op} {rhs}")
            }
        }
    }
}

impl BitAnd for SubfieldMatcher {
    type Output = Self;

    /// The bitwise AND operator `&` of two [SubfieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "foobar")?;
    ///
    /// let lhs = SubfieldMatcher::new("a =^ 'foo'")?;
    /// let rhs = SubfieldMatcher::new("a =$ 'bar'")?;
    /// let matcher = lhs & rhs;
    ///
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        let maybe_group = |m: Self| -> Self {
            match m {
                Self::Composite { ref op, .. }
                    if *op == BooleanOp::Or
                        || *op == BooleanOp::Xor =>
                {
                    Self::Group(Box::new(m.clone()))
                }
                _ => m,
            }
        };

        Self::Composite {
            lhs: Box::new(maybe_group(self)),
            op: BooleanOp::And,
            rhs: Box::new(maybe_group(rhs)),
        }
    }
}

impl BitAndAssign for SubfieldMatcher {
    /// The bitwise AND assignment operator `&=` of two
    /// [SubfieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "foobar")?;
    ///
    /// let mut matcher = SubfieldMatcher::new("a =^ 'foo'")?;
    /// matcher &= SubfieldMatcher::new("a =$ 'bar'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        let maybe_group = |m: &Self| -> Self {
            match m {
                Self::Composite { op, .. }
                    if *op == BooleanOp::Or
                        || *op == BooleanOp::Xor =>
                {
                    Self::Group(Box::new(m.clone()))
                }
                _ => m.clone(),
            }
        };

        *self = Self::Composite {
            lhs: Box::new(maybe_group(self)),
            op: BooleanOp::And,
            rhs: Box::new(maybe_group(&rhs)),
        }
    }
}

impl BitOr for SubfieldMatcher {
    type Output = Self;

    /// The bitwise OR operator `|` of two [SubfieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "bar")?;
    ///
    /// let lhs = SubfieldMatcher::new("a == 'foo'")?;
    /// let rhs = SubfieldMatcher::new("a == 'bar'")?;
    /// let matcher = lhs | rhs;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Composite {
            lhs: Box::new(self),
            op: BooleanOp::Or,
            rhs: Box::new(rhs),
        }
    }
}

impl BitOrAssign for SubfieldMatcher {
    /// The bitwise OR assignment operator `|=` of two
    /// [SubfieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "foo")?;
    ///
    /// let mut matcher = SubfieldMatcher::new("a == 'foo'")?;
    /// matcher |= SubfieldMatcher::new("a == 'bar'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self::Composite {
            lhs: Box::new(self.clone()),
            op: BooleanOp::Or,
            rhs: Box::new(rhs),
        }
    }
}

impl BitXor for SubfieldMatcher {
    type Output = Self;

    /// The bitwise XOR operator `^` of two [SubfieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "bar")?;
    ///
    /// let lhs = SubfieldMatcher::new("a == 'foo'")?;
    /// let rhs = SubfieldMatcher::new("a == 'bar'")?;
    /// let matcher = lhs ^ rhs;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        let maybe_group = |m: Self| -> Self {
            match m {
                Self::Composite { ref op, .. }
                    if *op == BooleanOp::Or =>
                {
                    Self::Group(Box::new(m.clone()))
                }
                _ => m,
            }
        };
        Self::Composite {
            lhs: Box::new(maybe_group(self)),
            op: BooleanOp::Xor,
            rhs: Box::new(maybe_group(rhs)),
        }
    }
}

impl BitXorAssign for SubfieldMatcher {
    /// The bitwise XOR assignment operator `^=` of two
    /// [SubfieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "foo")?;
    ///
    /// let mut matcher = SubfieldMatcher::new("a == 'foo'")?;
    /// matcher ^= SubfieldMatcher::new("a == 'bar'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        let maybe_group = |m: &Self| -> Self {
            match m {
                Self::Composite { op, .. } if *op == BooleanOp::Or => {
                    Self::Group(Box::new(m.clone()))
                }
                _ => m.clone(),
            }
        };

        *self = Self::Composite {
            lhs: Box::new(maybe_group(self)),
            op: BooleanOp::Xor,
            rhs: Box::new(maybe_group(&rhs)),
        }
    }
}

impl Not for SubfieldMatcher {
    type Output = Self;

    /// The unary logical negation operator `!` applied to a
    /// [SubfieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::SubfieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let subfield = SubfieldRef::new('a', "foo")?;
    ///
    /// let matcher = !SubfieldMatcher::new("a == 'bar'")?;
    /// assert!(matcher.is_match(&subfield, &options));
    ///
    /// let matcher = !SubfieldMatcher::new("a == 'foo'")?;
    /// assert!(!matcher.is_match(&subfield, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Self::Singleton(SingletonMatcher::Exists(_))
            | Self::Group(_)
            | Self::Not(_) => Self::Not(Box::new(self)),
            _ => Self::Not(Box::new(Self::Group(Box::new(self)))),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for SubfieldMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SubfieldMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn test_subfield_matcher_serde() -> TestResult {
        let matcher = SubfieldMatcher::new("a? && (b? || c?)")?;
        assert_tokens(&matcher, &[Token::Str("a? && (b? || c?)")]);

        let matcher = SubfieldMatcher::new("a? && (b? || c?) ")?;
        assert_tokens(&matcher, &[Token::Str("a? && (b? || c?)")]);
        Ok(())
    }

    #[test]
    fn test_cardinality_matcher() -> TestResult {
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let options = MatcherOptions::default();

        let matcher = CardinalityMatcher::new("#c == 0")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = CardinalityMatcher::new("#a != 1")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = CardinalityMatcher::new("#a >= 2")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = CardinalityMatcher::new("#a > 1")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = CardinalityMatcher::new("#b <= 2")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = CardinalityMatcher::new("#c < 1")?;
        assert!(matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_regex_matcher() -> TestResult {
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let options = MatcherOptions::default();
        let matcher = RegexMatcher::new("a =~ '^f.o$'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RegexMatcher::new("ANY a =~ '^f.o$'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RegexMatcher::new("ALL a =~ '^f.o$'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RegexMatcher::new("a !~ '^f.o$'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RegexMatcher::new("ANY a !~ '^f.o$'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RegexMatcher::new("ALL a !~ '^f.o$'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RegexMatcher::new("b !~ '^b.z$'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let options = MatcherOptions::default().case_ignore(true);
        let matcher = RegexMatcher::new("a =~ '^F.O$'")?;
        assert!(matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_regex_set_matcher() -> TestResult {
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let options = MatcherOptions::default();
        let matcher = RegexSetMatcher::new("a =~ ['^f.o$', '^bar']")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher =
            RegexSetMatcher::new("ANY a =~ ['^f.o$', '^bar']")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher =
            RegexSetMatcher::new("ALL a =~ ['^f.o$', '^bar']")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RegexSetMatcher::new("a !~ ['^f.o$', '^bar']")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher =
            RegexSetMatcher::new("ANY a !~ ['^f.o$', '^bar']")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher =
            RegexSetMatcher::new("ALL a !~ ['^f.o$', '^bar']")?;
        assert!(!matcher.is_match(&subfields, &options));

        let options = MatcherOptions::default().case_ignore(true);
        let matcher = RegexSetMatcher::new("a =~ ['^F.O$', '^BAR']")?;
        assert!(matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_relation_matcher() -> TestResult {
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let options = MatcherOptions::default();
        let matcher = RelationMatcher::new("a == 'foo'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a == 'baz'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a != 'foo'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("b != 'baz'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a =^ 'fo'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a =^ 'FO'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a !^ 'fo'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("b !^ 'b'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a =$ 'o'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a =$ 'O'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a !$ 'o'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("b !$ 'z'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a =* 'foo'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a =* 'foO'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a =? 'oo'")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = RelationMatcher::new("a =? 'frob'")?;
        assert!(!matcher.is_match(&subfields, &options));

        let options = MatcherOptions::default().case_ignore(true);
        let matcher = RelationMatcher::new("a == 'FOO'")?;
        assert!(matcher.is_match(&subfields, &options));

        let options = MatcherOptions::default().case_ignore(true);
        let matcher = RelationMatcher::new("a =* 'foO'")?;
        assert!(matcher.is_match(&subfields, &options));

        let options = MatcherOptions::default().strsim_threshold(0.65);
        let matcher = RelationMatcher::new("a =* 'foO'")?;
        assert!(matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_exists_matcher() -> TestResult {
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let options = MatcherOptions::default();
        let matcher = ExistsMatcher::new("a?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = ExistsMatcher::new("c?")?;
        assert!(!matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_in_matcher() -> TestResult {
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let options = MatcherOptions::default();
        let matcher = InMatcher::new("a in ['foo', 'baz']")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = InMatcher::new("a in ['frob', 'baz']")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = InMatcher::new("a not in ['foo', 'baz']")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = InMatcher::new("b not in ['frob', 'baz']")?;
        assert!(!matcher.is_match(&subfields, &options));

        let options = MatcherOptions::default().case_ignore(true);
        let matcher = InMatcher::new("a in ['FOO', 'BAZ']")?;
        assert!(matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_group() -> TestResult {
        let options = MatcherOptions::default();
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('b', "bar")?,
            SubfieldRef::new('c', "baz")?,
        ];

        let matcher = SubfieldMatcher::new("(a?)")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("(a? && b == 'bar')")?;
        assert!(matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_not() -> TestResult {
        let options = MatcherOptions::default();
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('b', "bar")?,
            SubfieldRef::new('c', "baz")?,
        ];

        let matcher = SubfieldMatcher::new("!d?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("!a?")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("!(d? || a?)")?;
        assert!(!matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_and() -> TestResult {
        let options = MatcherOptions::default();
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let matcher = SubfieldMatcher::new("a? && b?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("a? && b? && c?")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("a? && b? || c?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("a? || b? && c?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("c? || b? && c?")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("b? && c? || c?")?;
        assert!(!matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_or() -> TestResult {
        let options = MatcherOptions::default();
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let matcher = SubfieldMatcher::new("a? || b?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("a? || b? || c?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("(a? && b?) || c?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("a? || (b? && c?)")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("(a? || b?) && c? || d?")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("a? && (b? || c?) || d?")?;
        assert!(matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_xor() -> TestResult {
        let options = MatcherOptions::default();
        let subfields = vec![
            SubfieldRef::new('a', "foo")?,
            SubfieldRef::new('a', "bar")?,
            SubfieldRef::new('b', "baz")?,
        ];

        let matcher = SubfieldMatcher::new("a? ^ b?")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("a? ^ b? ^ c?")?;
        assert!(!matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("a? ^ c?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("c? ^ a?")?;
        assert!(matcher.is_match(&subfields, &options));

        let matcher = SubfieldMatcher::new("c? ^ d?")?;
        assert!(!matcher.is_match(&subfields, &options));

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_bitand() -> TestResult {
        let expected = SubfieldMatcher::new("a? && b?")?;
        let lhs = SubfieldMatcher::new("a?")?;
        let rhs = SubfieldMatcher::new("b?")?;
        assert_eq!(lhs & rhs, expected);

        let expected = SubfieldMatcher::new("(a? || b?) && c?")?;
        let lhs = SubfieldMatcher::new("a? || b?")?;
        let rhs = SubfieldMatcher::new("c?")?;
        assert_eq!(lhs & rhs, expected);

        let expected = SubfieldMatcher::new("a? && (b? || c?)")?;
        let lhs = SubfieldMatcher::new("a?")?;
        let rhs = SubfieldMatcher::new("(b? || c?)")?;
        assert_eq!(lhs & rhs, expected);

        let expected = SubfieldMatcher::new("(a? ^ b?) && c?")?;
        let lhs = SubfieldMatcher::new("a? ^ b?")?;
        let rhs = SubfieldMatcher::new("c?")?;
        assert_eq!(lhs & rhs, expected);

        let expected = SubfieldMatcher::new("a? && (b? ^ c?)")?;
        let lhs = SubfieldMatcher::new("a?")?;
        let rhs = SubfieldMatcher::new("(b? ^ c?)")?;
        assert_eq!(lhs & rhs, expected);

        let expected = SubfieldMatcher::new("a? && b? && c?")?;
        let lhs = SubfieldMatcher::new("a? && b?")?;
        let rhs = SubfieldMatcher::new("c?")?;
        assert_eq!(lhs & rhs, expected);

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_bitand_assign() -> TestResult {
        let expected = SubfieldMatcher::new("a? && b?")?;
        let mut matcher = SubfieldMatcher::new("a?")?;
        matcher &= SubfieldMatcher::new("b?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("(a? || b?) && c?")?;
        let mut matcher = SubfieldMatcher::new("a? || b?")?;
        matcher &= SubfieldMatcher::new("c?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("a? && (b? || c?)")?;
        let mut matcher = SubfieldMatcher::new("a?")?;
        matcher &= SubfieldMatcher::new("(b? || c?)")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("(a? ^ b?) && c?")?;
        let mut matcher = SubfieldMatcher::new("a? ^ b?")?;
        matcher &= SubfieldMatcher::new("c?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("a? && (b? ^ c?)")?;
        let mut matcher = SubfieldMatcher::new("a?")?;
        matcher &= SubfieldMatcher::new("(b? ^ c?)")?;
        assert_eq!(matcher, expected);

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_bitxor() -> TestResult {
        let expected = SubfieldMatcher::new("a? ^ b?")?;
        let lhs = SubfieldMatcher::new("a?")?;
        let rhs = SubfieldMatcher::new("b?")?;
        assert_eq!(lhs ^ rhs, expected);

        let expected = SubfieldMatcher::new("(a? || b?) ^ c?")?;
        let lhs = SubfieldMatcher::new("a? || b?")?;
        let rhs = SubfieldMatcher::new("c?")?;
        assert_eq!(lhs ^ rhs, expected);

        let expected = SubfieldMatcher::new("a? ^ (b? || c?)")?;
        let lhs = SubfieldMatcher::new("a?")?;
        let rhs = SubfieldMatcher::new("(b? || c?)")?;
        assert_eq!(lhs ^ rhs, expected);

        let expected = SubfieldMatcher::new("a? ^ b? ^ c?")?;
        let lhs = SubfieldMatcher::new("a? ^ b?")?;
        let rhs = SubfieldMatcher::new("c?")?;
        assert_eq!(lhs ^ rhs, expected);

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_bitxor_assign() -> TestResult {
        let expected = SubfieldMatcher::new("a? ^ b?")?;
        let mut matcher = SubfieldMatcher::new("a?")?;
        matcher ^= SubfieldMatcher::new("b?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("(a? || b?) ^ c?")?;
        let mut matcher = SubfieldMatcher::new("a? || b?")?;
        matcher ^= SubfieldMatcher::new("c?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("a? ^ (b? || c?)")?;
        let mut matcher = SubfieldMatcher::new("a?")?;
        matcher ^= SubfieldMatcher::new("(b? || c?)")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("a? ^ b? ^ c?")?;
        let mut matcher = SubfieldMatcher::new("a? ^ b?")?;
        matcher ^= SubfieldMatcher::new("c?")?;
        assert_eq!(matcher, expected);

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_bitor() -> TestResult {
        let expected = SubfieldMatcher::new("a? || b?")?;
        let lhs = SubfieldMatcher::new("a?")?;
        let rhs = SubfieldMatcher::new("b?")?;
        assert_eq!(lhs | rhs, expected);

        let expected = SubfieldMatcher::new("a? || b? || c?")?;
        let lhs = SubfieldMatcher::new("a? || b?")?;
        let rhs = SubfieldMatcher::new("c?")?;
        assert_eq!(lhs | rhs, expected);

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_bitor_assign() -> TestResult {
        let expected = SubfieldMatcher::new("a? || b?")?;
        let mut matcher = SubfieldMatcher::new("a?")?;
        matcher |= SubfieldMatcher::new("b?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("a? || b? || c?")?;
        let mut matcher = SubfieldMatcher::new("a? || b?")?;
        matcher |= SubfieldMatcher::new("c?")?;
        assert_eq!(matcher, expected);

        Ok(())
    }

    #[test]
    fn test_subfield_matcher_bitnot() -> TestResult {
        let expected = SubfieldMatcher::new("!(a?)")?;
        let matcher = !SubfieldMatcher::new("(a?)")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("!a?")?;
        let matcher = !SubfieldMatcher::new("a?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("!!a?")?;
        let matcher = !SubfieldMatcher::new("!a?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("!(a == 'foo')")?;
        let matcher = !SubfieldMatcher::new("a == 'foo'")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("!(a? && b?)")?;
        let matcher = !SubfieldMatcher::new("a? && b?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("!(a? || b?)")?;
        let matcher = !SubfieldMatcher::new("a? || b?")?;
        assert_eq!(matcher, expected);

        let expected = SubfieldMatcher::new("!(a? ^ b?)")?;
        let matcher = !SubfieldMatcher::new("a? ^ b?")?;
        assert_eq!(matcher, expected);

        Ok(())
    }
}
