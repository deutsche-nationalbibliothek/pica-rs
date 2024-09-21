//! Matcher that can be applied on a list of [FieldRef].

use std::fmt::{self, Display};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not,
};

use parser::{
    parse_cardinality_matcher, parse_exists_matcher,
    parse_field_matcher, parse_singleton_matcher,
    parse_subfields_matcher,
};
use winnow::Parser;

use super::{
    subfield, BooleanOp, MatcherOptions, OccurrenceMatcher,
    ParseMatcherError, Quantifier, RelationalOp, TagMatcher,
};
use crate::prelude::SubfieldMatcher;
use crate::primitives::FieldRef;

pub(crate) mod parser;

/// A matcher that checks if a field exists.
#[derive(Debug, Clone, PartialEq)]
pub struct ExistsMatcher {
    tag_matcher: TagMatcher,
    occ_matcher: OccurrenceMatcher,
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
    /// use pica_record::matcher::field::ExistsMatcher;
    ///
    /// let _matcher = ExistsMatcher::new("003@?")?;
    /// let _matcher = ExistsMatcher::new("041[A@]?")?;
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

    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::ExistsMatcher;
    /// use pica_record::matcher::MatcherOptions;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "123456789X")])?;
    /// let options = MatcherOptions::default();
    /// let matcher = ExistsMatcher::new("003@?")?;
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>>,
        _options: &MatcherOptions,
    ) -> bool {
        fields.into_iter().any(|field| {
            self.tag_matcher.is_match(field.tag())
                && self.occ_matcher.is_match(field.occurrence())
        })
    }
}

impl Display for ExistsMatcher {
    /// Formats an [ExistsMatcher] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::ExistsMatcher;
    ///
    /// let matcher = ExistsMatcher::new("..../*?")?;
    /// assert_eq!(matcher.to_string(), "..../*?");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}?", self.tag_matcher, self.occ_matcher)
    }
}

/// A matcher that checks whether the subfields meet a criterion.
#[derive(Debug, Clone, PartialEq)]
pub struct SubfieldsMatcher {
    quantifier: Quantifier,
    tag_matcher: TagMatcher,
    occurrence_matcher: OccurrenceMatcher,
    subfield_matcher: SubfieldMatcher,
    raw_data: String,
}

impl SubfieldsMatcher {
    /// Creates a new [SubfieldsMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// subfields matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::SubfieldsMatcher;
    ///
    /// let _matcher = SubfieldsMatcher::new("003@.0 == '0123456789X'")?;
    /// let _matcher =
    ///     SubfieldsMatcher::new("002@{0 == 'Oaf' || 0 == 'Olfo'}")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_subfields_matcher.parse(matcher.as_bytes()).map_err(
            |_| {
                ParseMatcherError(format!(
                    "invalid subfields matcher '{matcher}'"
                ))
            },
        )
    }

    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::SubfieldsMatcher;
    /// use pica_record::matcher::MatcherOptions;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "123456789X")])?;
    /// let options = MatcherOptions::default();
    /// let matcher = SubfieldsMatcher::new("003@.0 == '123456789X'")?;
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let mut fields = fields.into_iter().filter(|field| {
            self.tag_matcher.is_match(field.tag())
                && self.occurrence_matcher.is_match(field.occurrence())
        });

        let r#fn = |field: &FieldRef| -> bool {
            self.subfield_matcher.is_match(field.subfields(), options)
        };

        match self.quantifier {
            Quantifier::All => fields.all(r#fn),
            Quantifier::Any => fields.any(r#fn),
        }
    }
}

impl Display for SubfieldsMatcher {
    /// Formats an [SubfieldsMatcher] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::SubfieldsMatcher;
    ///
    /// let matcher = SubfieldsMatcher::new("..../*.* == 'foo'")?;
    /// assert_eq!(matcher.to_string(), "..../*.* == 'foo'");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_data)
    }
}

/// A matcher that checks for an [ExistsMatcher] or a
/// [SubfieldsMatcher].
#[derive(Debug, Clone, PartialEq)]
pub enum SingletonMatcher {
    Subfields(SubfieldsMatcher),
    Exists(ExistsMatcher),
}

impl SingletonMatcher {
    /// Creates a new [SingletonMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// singleton matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::SingletonMatcher;
    ///
    /// let _matcher = SingletonMatcher::new("003@.0 == '0123456789X'")?;
    /// let _matcher = SingletonMatcher::new("041A/*?")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_singleton_matcher.parse(matcher.as_bytes()).map_err(
            |_| {
                ParseMatcherError(format!(
                    "invalid singleton matcher '{matcher}'"
                ))
            },
        )
    }

    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::SingletonMatcher;
    /// use pica_record::matcher::MatcherOptions;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "123456789X")])?;
    /// let options = MatcherOptions::default();
    ///
    /// let matcher = SingletonMatcher::new("003@.0 == '123456789X'")?;
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// let matcher = SingletonMatcher::new("003@?")?;
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        match self {
            Self::Subfields(m) => m.is_match(fields, options),
            Self::Exists(m) => m.is_match(fields, options),
        }
    }
}

impl Display for SingletonMatcher {
    /// Formats an [SingletonMatcher] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::SingletonMatcher;
    ///
    /// let matcher = SingletonMatcher::new("..../*.* == 'foo'")?;
    /// assert_eq!(matcher.to_string(), "..../*.* == 'foo'");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subfields(m) => write!(f, "{m}"),
            Self::Exists(m) => write!(f, "{m}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CardinalityMatcher {
    tag_matcher: TagMatcher,
    occ_matcher: OccurrenceMatcher,
    subfield_matcher: Option<SubfieldMatcher>,
    op: RelationalOp,
    value: usize,
    raw_data: String,
}

impl CardinalityMatcher {
    /// Creates a new [CardinalityMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// singleton matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::CardinalityMatcher;
    ///
    /// let _matcher = CardinalityMatcher::new("#010@{a == 'ger'} > 1")?;
    /// let _matcher = CardinalityMatcher::new("#003@ > 1")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_cardinality_matcher.parse(matcher.as_bytes()).map_err(
            |_| {
                ParseMatcherError(format!(
                    "invalid cardinality matcher '{matcher}'"
                ))
            },
        )
    }

    /// Returns `true` if the matcher matches against the given
    /// subfield(s).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::CardinalityMatcher;
    /// use pica_record::matcher::MatcherOptions;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field = FieldRef::new("003@", None, vec![('0', "123456789X")])?;
    /// let options = MatcherOptions::default();
    ///
    /// let matcher = CardinalityMatcher::new("#003@ == 1")?;
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>>,
        options: &MatcherOptions,
    ) -> bool {
        let count = fields
            .into_iter()
            .filter(|field| {
                let retval = self.tag_matcher.is_match(field.tag())
                    && self.occ_matcher.is_match(field.occurrence());

                if let Some(ref matcher) = self.subfield_matcher {
                    retval
                        && matcher.is_match(field.subfields(), options)
                } else {
                    retval
                }
            })
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
    /// Formats an [CardinalityMatcher] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::CardinalityMatcher;
    ///
    /// let matcher = CardinalityMatcher::new("#010@{ a == 'ger' } > 1")?;
    /// assert_eq!(matcher.to_string(), "#010@{ a == 'ger' } > 1");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_data)
    }
}

/// A matcher that allows grouping, negation and connecting of
/// [SingletonMatcher].
#[derive(Debug, Clone, PartialEq)]
pub enum FieldMatcher {
    Singleton(SingletonMatcher),
    Cardinality(CardinalityMatcher),
    Group(Box<FieldMatcher>),
    Not(Box<FieldMatcher>),
    Composite {
        lhs: Box<FieldMatcher>,
        op: BooleanOp,
        rhs: Box<FieldMatcher>,
    },
}

impl FieldMatcher {
    /// Creates a new [FieldMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// field matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::FieldMatcher;
    ///
    /// let _matcher = FieldMatcher::new("#003@ > 1")?;
    /// let _matcher = FieldMatcher::new("010@.a == 'ger'")?;
    /// let _matcher = FieldMatcher::new("(010@.a == 'ger')")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        parse_field_matcher.parse(matcher.as_bytes()).map_err(|_| {
            ParseMatcherError(format!(
                "invalid field matcher '{matcher}'"
            ))
        })
    }

    /// Returns `true` if the given field(s) matches against the field
    /// matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::FieldMatcher;
    /// use pica_record::matcher::MatcherOptions;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let field =
    ///     FieldRef::new("003@", None, vec![('0', "0123456789X")])?;
    ///
    /// let options = MatcherOptions::default();
    /// let matcher = FieldMatcher::new("#003@ == 1")?;
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_match<'a>(
        &self,
        fields: impl IntoIterator<Item = &'a FieldRef<'a>> + Clone,
        options: &MatcherOptions,
    ) -> bool {
        use BooleanOp::*;

        match self {
            Self::Singleton(m) => m.is_match(fields, options),
            Self::Cardinality(m) => m.is_match(fields, options),
            Self::Group(m) => m.is_match(fields, options),
            Self::Not(m) => !m.is_match(fields, options),
            Self::Composite { lhs, op, rhs } => {
                let lhs = lhs.is_match(fields.clone(), options);
                match *op {
                    Xor => lhs != rhs.is_match(fields, options),
                    And => lhs && rhs.is_match(fields, options),
                    Or => lhs || rhs.is_match(fields, options),
                }
            }
        }
    }
}

impl Display for FieldMatcher {
    /// Format the subfield-matcher as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::field::FieldMatcher;
    ///
    /// let matcher = FieldMatcher::new("#012A > 3")?;
    /// assert_eq!(matcher.to_string(), "#012A > 3");
    ///
    /// let mut matcher =
    ///     FieldMatcher::new("012A.a == 'foo' || 012A.a == 'bar'")?;
    /// matcher &= FieldMatcher::new("012A.c == 'baz'")?;
    /// assert_eq!(
    ///     matcher.to_string(),
    ///     "(012A.a == 'foo' || 012A.a == 'bar') && 012A.c == 'baz'"
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cardinality(m) => write!(f, "{m}"),
            Self::Singleton(m) => write!(f, "{m}"),
            Self::Group(m) => write!(f, "({m})"),
            Self::Not(m) => write!(f, "!{m}"),
            Self::Composite { lhs, op, rhs } => {
                write!(f, "{lhs} {op} {rhs}")
            }
        }
    }
}

impl BitAnd for FieldMatcher {
    type Output = Self;

    /// The bitwise AND operator `&` of two [FieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let field =
    ///     FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?;
    ///
    /// let lhs = FieldMatcher::new("012A.a == 'A'")?;
    /// let rhs = FieldMatcher::new("012A.b == 'B'")?;
    /// let matcher = lhs & rhs;
    ///
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        let group = |m: Self| -> Self {
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
            lhs: Box::new(group(self)),
            op: BooleanOp::And,
            rhs: Box::new(group(rhs)),
        }
    }
}

impl BitAndAssign for FieldMatcher {
    /// The bitwise AND assignment operator `&=` of two [FieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let field =
    ///     FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?;
    ///
    /// let mut matcher = FieldMatcher::new("012A.a == 'A'")?;
    /// matcher &= FieldMatcher::new("012A.b == 'B'")?;
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        let group = |m: &Self| -> Self {
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
            lhs: Box::new(group(self)),
            op: BooleanOp::And,
            rhs: Box::new(group(&rhs)),
        }
    }
}

impl BitOr for FieldMatcher {
    type Output = Self;

    /// The bitwise OR operator `|` of two [FieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let field =
    ///     FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?;
    ///
    /// let lhs = FieldMatcher::new("012A.a == 'X'")?;
    /// let rhs = FieldMatcher::new("012A.b == 'B'")?;
    /// let matcher = lhs | rhs;
    ///
    /// assert!(matcher.is_match(&field, &options));
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

impl BitOrAssign for FieldMatcher {
    /// The bitwise OR assignment operator `|=` of two [FieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let field =
    ///     FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?;
    ///
    /// let mut matcher = FieldMatcher::new("012A.a == 'X'")?;
    /// matcher |= FieldMatcher::new("012A.b == 'B'")?;
    /// assert!(matcher.is_match(&field, &options));
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

impl BitXor for FieldMatcher {
    type Output = Self;

    /// The bitwise XOR operator `^` of two [FieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let field =
    ///     FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?;
    ///
    /// let lhs = FieldMatcher::new("012A.a == 'X'")?;
    /// let rhs = FieldMatcher::new("012A.b == 'B'")?;
    /// let matcher = lhs ^ rhs;
    ///
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        let group = |m: Self| -> Self {
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
            lhs: Box::new(group(self)),
            op: BooleanOp::Xor,
            rhs: Box::new(group(rhs)),
        }
    }
}

impl BitXorAssign for FieldMatcher {
    /// The bitwise XOR assignment operator `^=` of two [FieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let field =
    ///     FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?;
    ///
    /// let mut matcher = FieldMatcher::new("012A.a == 'X'")?;
    /// matcher ^= FieldMatcher::new("012A.b == 'B'")?;
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        let group = |m: &Self| -> Self {
            match m {
                Self::Composite { op, .. } if *op == BooleanOp::Or => {
                    Self::Group(Box::new(m.clone()))
                }
                _ => m.clone(),
            }
        };

        *self = Self::Composite {
            lhs: Box::new(group(self)),
            op: BooleanOp::Xor,
            rhs: Box::new(group(&rhs)),
        }
    }
}

impl Not for FieldMatcher {
    type Output = Self;

    /// The unary logical negation operator `!` applied to a
    /// [FieldMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::FieldRef;
    ///
    /// let options = MatcherOptions::default();
    /// let field =
    ///     FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?;
    ///
    /// let matcher = FieldMatcher::new("012A.a == 'X'")?;
    /// let matcher = !matcher;
    ///
    /// assert!(matcher.is_match(&field, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Self::Singleton(SingletonMatcher::Subfields(
                SubfieldsMatcher {
                    subfield_matcher:
                        subfield::SubfieldMatcher::Singleton(
                            subfield::SingletonMatcher::Exists(_),
                        ),
                    ..
                },
            ))
            | Self::Group(_)
            | Self::Not(_) => Self::Not(Box::new(self)),
            _ => Self::Not(Box::new(Self::Group(Box::new(self)))),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for FieldMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FieldMatcher {
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
    use crate::primitives::FieldRef;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn test_field_matcher_serde() -> TestResult {
        let matcher =
            FieldMatcher::new("012A.a? && (012A.b? || 012A.c?)")?;
        assert_tokens(
            &matcher,
            &[Token::Str("012A.a? && (012A.b? || 012A.c?)")],
        );

        let matcher =
            FieldMatcher::new("012A.a? && !(012A.b? || 012A.c?) ")?;
        assert_tokens(
            &matcher,
            &[Token::Str("012A.a? && !(012A.b? || 012A.c?)")],
        );
        Ok(())
    }

    #[test]
    fn test_exists_matcher() -> TestResult {
        let options = MatcherOptions::default();
        let field = FieldRef::new("012A", None, vec![('a', "abc")]);

        let matcher = ExistsMatcher::new("012A?")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = ExistsMatcher::new("013A?")?;
        assert!(!matcher.is_match(&field, &options));

        Ok(())
    }

    #[test]
    fn test_singleton_matcher() -> TestResult {
        let options = MatcherOptions::default();
        let field = FieldRef::new(
            "012A",
            None,
            vec![('a', "abc"), ('a', "def"), ('b', "hij")],
        );

        let matcher = SingletonMatcher::new("012A.a == 'def'")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = SingletonMatcher::new("012A.a == 'hij'")?;
        assert!(!matcher.is_match(&field, &options));

        let matcher =
            SingletonMatcher::new("012A{ b? && a == 'def' }")?;
        assert!(matcher.is_match(&field, &options));

        let field =
            FieldRef::new("012A", Some("03"), vec![('a', "abc")]);

        let matcher = SingletonMatcher::new("012A/03.a == 'abc'")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = SingletonMatcher::new("012A/*.a == 'abc'")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = SingletonMatcher::new("012A/02-04.a == 'abc'")?;
        assert!(matcher.is_match(&field, &options));

        Ok(())
    }

    #[test]
    fn test_cardinality_matcher() -> TestResult {
        let options = MatcherOptions::default();
        let field = FieldRef::new(
            "012A",
            None,
            vec![('a', "abc"), ('a', "def"), ('b', "hij")],
        );

        let matcher = CardinalityMatcher::new("#013A == 0")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A == 1")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A != 2")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A >= 1")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A > 0")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A <= 100")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A < 100")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A{ c? } == 0")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A{ b? } == 1")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A{ b? } != 2")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A{ b? } >= 1")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A{ b? } > 0")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A{ b? } <= 1")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = CardinalityMatcher::new("#012A{ b? } < 100")?;
        assert!(matcher.is_match(&field, &options));

        Ok(())
    }

    #[test]
    fn test_field_matcher_singleton() -> TestResult {
        let options = MatcherOptions::default();
        let field =
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")]);

        let matcher = FieldMatcher::new("012A.a == 'A'")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = FieldMatcher::new("012A.[a-d] == 'B'")?;
        assert!(matcher.is_match(&field, &options));

        let options = MatcherOptions::default();
        let fields = vec![
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?,
            FieldRef::new("012A", None, vec![('c', "C"), ('d', "D")])?,
        ];

        let matcher = FieldMatcher::new("012A.[df] == 'D'")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("ALL 012A.[a-d] in ['B', 'D']")?;
        assert!(matcher.is_match(&fields, &options));

        Ok(())
    }

    #[test]
    fn test_field_matcher_cardinality() -> TestResult {
        let options = MatcherOptions::default();
        let field =
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")]);

        let matcher = FieldMatcher::new("#012A{a == 'A'} == 1")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = FieldMatcher::new("#012A{ [a-d] == 'B'} >= 1")?;
        assert!(matcher.is_match(&field, &options));

        let options = MatcherOptions::default();
        let fields = vec![
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?,
            FieldRef::new("012A", None, vec![('c', "C"), ('d', "D")])?,
        ];

        let matcher = FieldMatcher::new("#012A{ [df] == 'D' } == 1")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("#012A{ [a-d] in ['B', 'D']} <= 2")?;
        assert!(matcher.is_match(&fields, &options));

        Ok(())
    }

    #[test]
    fn test_field_matcher_group() -> TestResult {
        let options = MatcherOptions::default();
        let field =
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")]);

        let matcher = FieldMatcher::new("(012A.a?)")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = FieldMatcher::new("((012A.b?))")?;
        assert!(matcher.is_match(&field, &options));

        Ok(())
    }

    #[test]
    fn test_field_matcher_not() -> TestResult {
        let options = MatcherOptions::default();
        let field =
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")]);

        let matcher = FieldMatcher::new("!(012A.c?)")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = FieldMatcher::new("!012A{ a? && c? }")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = FieldMatcher::new("!012A.c?")?;
        assert!(matcher.is_match(&field, &options));

        let matcher = FieldMatcher::new("!!012A.a?")?;
        assert!(matcher.is_match(&field, &options));

        Ok(())
    }

    #[test]
    fn test_field_matcher_and() -> TestResult {
        let options = MatcherOptions::default();
        let fields = vec![
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?,
            FieldRef::new("013A", None, vec![('c', "C")])?,
        ];

        let matcher = FieldMatcher::new("012A.a? && 013A.c?")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("012A.a? && 012A.b? && 012A.c?")?;
        assert!(!matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("012A.a? && 012A.b? || 014A.c?")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("014A.c? || 012A.b? && 013A.c?")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("012A.c? || 012A.b? && 012A.c?")?;
        assert!(!matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("012A.b? && 012A.c? || 012A.c?")?;
        assert!(!matcher.is_match(&fields, &options));

        Ok(())
    }

    #[test]
    fn test_field_matcher_or() -> TestResult {
        let options = MatcherOptions::default();
        let fields = vec![
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?,
            FieldRef::new("013A", None, vec![('c', "C")])?,
        ];

        let matcher = FieldMatcher::new("013A.a? || 012A.b?")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("014A.a? || 013A.b? || 012A.b?")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("(012A.a? && 012A.b?) || 012A.c?")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher =
            FieldMatcher::new("013A.a? || (012A.b? && 013A.c?)")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher = FieldMatcher::new(
            "(012A.a? || 012A.b?) && 012A.c? || 013A.d?",
        )?;
        assert!(!matcher.is_match(&fields, &options));

        let matcher = FieldMatcher::new(
            "012A.a? && (012A.b? || 012A.c?) || 013A.c?",
        )?;
        assert!(matcher.is_match(&fields, &options));

        Ok(())
    }

    #[test]
    fn test_field_matcher_xor() -> TestResult {
        let options = MatcherOptions::default();
        let fields = vec![
            FieldRef::new("012A", None, vec![('a', "A"), ('b', "B")])?,
            FieldRef::new("013A", None, vec![('c', "C")])?,
        ];

        let matcher = FieldMatcher::new("012A.a? ^ 012A.b?")?;
        assert!(!matcher.is_match(&fields, &options));

        let matcher = FieldMatcher::new("012A.a? ^ 012A.b? ^ 012A.c?")?;
        assert!(!matcher.is_match(&fields, &options));

        let matcher = FieldMatcher::new("012A.a? ^ 012A.c?")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher = FieldMatcher::new("012A.c? ^ 012A.a?")?;
        assert!(matcher.is_match(&fields, &options));

        let matcher = FieldMatcher::new("012A.c? ^ 012A.d?")?;
        assert!(!matcher.is_match(&fields, &options));

        Ok(())
    }

    #[test]
    fn test_field_matcher_bitand() -> TestResult {
        let expected = FieldMatcher::new("012A.a? && 012A.b?")?;
        let lhs = FieldMatcher::new("012A.a?")?;
        let rhs = FieldMatcher::new("012A.b?")?;
        assert_eq!(lhs & rhs, expected);

        let expected =
            FieldMatcher::new("(012A.a? || 012A.b?) && 012A.c?")?;
        let lhs = FieldMatcher::new("012A.a? || 012A.b?")?;
        let rhs = FieldMatcher::new("012A.c?")?;
        assert_eq!(lhs & rhs, expected);

        let expected =
            FieldMatcher::new("012A.a? && (012A.b? || 012A.c?)")?;
        let lhs = FieldMatcher::new("012A.a?")?;
        let rhs = FieldMatcher::new("(012A.b? || 012A.c?)")?;
        assert_eq!(lhs & rhs, expected);

        let expected =
            FieldMatcher::new("(012A.a? ^ 012A.b?) && 012A.c?")?;
        let lhs = FieldMatcher::new("012A.a? ^ 012A.b?")?;
        let rhs = FieldMatcher::new("012A.c?")?;
        assert_eq!(lhs & rhs, expected);

        let expected =
            FieldMatcher::new("012A.a? && (012A.b? ^ 012A.c?)")?;
        let lhs = FieldMatcher::new("012A.a?")?;
        let rhs = FieldMatcher::new("(012A.b? ^ 012A.c?)")?;
        assert_eq!(lhs & rhs, expected);

        let expected =
            FieldMatcher::new("012A.a? && 012A.b? && 012A.c?")?;
        let lhs = FieldMatcher::new("012A.a? && 012A.b?")?;
        let rhs = FieldMatcher::new("012A.c?")?;
        assert_eq!(lhs & rhs, expected);

        Ok(())
    }

    #[test]
    fn test_field_matcher_bitand_assign() -> TestResult {
        let expected = FieldMatcher::new("012A.a? && 012A.b?")?;
        let mut matcher = FieldMatcher::new("012A.a?")?;
        matcher &= FieldMatcher::new("012A.b?")?;
        assert_eq!(matcher, expected);

        let expected =
            FieldMatcher::new("(012A.a? || 012A.b?) && 012A.c?")?;
        let mut matcher = FieldMatcher::new("012A.a? || 012A.b?")?;
        matcher &= FieldMatcher::new("012A.c?")?;
        assert_eq!(matcher, expected);

        let expected =
            FieldMatcher::new("012A.a? && (012A.b? || 012A.c?)")?;
        let mut matcher = FieldMatcher::new("012A.a?")?;
        matcher &= FieldMatcher::new("(012A.b? || 012A.c?)")?;
        assert_eq!(matcher, expected);

        let expected =
            FieldMatcher::new("(012A.a? ^ 012A.b?) && 012A.c?")?;
        let mut matcher = FieldMatcher::new("012A.a? ^ 012A.b?")?;
        matcher &= FieldMatcher::new("012A.c?")?;
        assert_eq!(matcher, expected);

        let expected =
            FieldMatcher::new("012A.a? && (012A.b? ^ 012A.c?)")?;
        let mut matcher = FieldMatcher::new("012A.a?")?;
        matcher &= FieldMatcher::new("(012A.b? ^ 012A.c?)")?;
        assert_eq!(matcher, expected);

        Ok(())
    }

    #[test]
    fn test_field_matcher_bitxor() -> TestResult {
        let expected = FieldMatcher::new("012A.a? ^ 012A.b?")?;
        let lhs = FieldMatcher::new("012A.a?")?;
        let rhs = FieldMatcher::new("012A.b?")?;
        assert_eq!(lhs ^ rhs, expected);

        let expected =
            FieldMatcher::new("(012A.a? || 012A.b?) ^ 012A.c?")?;
        let lhs = FieldMatcher::new("012A.a? || 012A.b?")?;
        let rhs = FieldMatcher::new("012A.c?")?;
        assert_eq!(lhs ^ rhs, expected);

        let expected =
            FieldMatcher::new("012A.a? ^ (012A.b? || 012A.c?)")?;
        let lhs = FieldMatcher::new("012A.a?")?;
        let rhs = FieldMatcher::new("(012A.b? || 012A.c?)")?;
        assert_eq!(lhs ^ rhs, expected);

        let expected =
            FieldMatcher::new("012A.a? ^ 012A.b? ^ 012A.c?")?;
        let lhs = FieldMatcher::new("012A.a? ^ 012A.b?")?;
        let rhs = FieldMatcher::new("012A.c?")?;
        assert_eq!(lhs ^ rhs, expected);

        Ok(())
    }

    #[test]
    fn test_field_matcher_bitxor_assign() -> TestResult {
        let expected = FieldMatcher::new("012A.a? ^ 012A.b?")?;
        let mut matcher = FieldMatcher::new("012A.a?")?;
        matcher ^= FieldMatcher::new("012A.b?")?;
        assert_eq!(matcher, expected);

        let expected =
            FieldMatcher::new("(012A.a? || 012A.b?) ^ 012A.c?")?;
        let mut matcher = FieldMatcher::new("012A.a? || 012A.b?")?;
        matcher ^= FieldMatcher::new("012A.c?")?;
        assert_eq!(matcher, expected);

        let expected =
            FieldMatcher::new("012A.a? ^ (012A.b? || 012A.c?)")?;
        let mut matcher = FieldMatcher::new("012A.a?")?;
        matcher ^= FieldMatcher::new("(012A.b? || 012A.c?)")?;
        assert_eq!(matcher, expected);

        let expected =
            FieldMatcher::new("012A.a? ^ 012A.b? ^ 012A.c?")?;
        let mut matcher = FieldMatcher::new("012A.a? ^ 012A.b?")?;
        matcher ^= FieldMatcher::new("012A.c?")?;
        assert_eq!(matcher, expected);

        Ok(())
    }

    #[test]
    fn test_field_matcher_bitor() -> TestResult {
        let expected = FieldMatcher::new("012A.a? || 012A.b?")?;
        let lhs = FieldMatcher::new("012A.a?")?;
        let rhs = FieldMatcher::new("012A.b?")?;
        assert_eq!(lhs | rhs, expected);

        let expected =
            FieldMatcher::new("012A.a? || 012A.b? || 012A.c?")?;
        let lhs = FieldMatcher::new("012A.a? || 012A.b?")?;
        let rhs = FieldMatcher::new("012A.c?")?;
        assert_eq!(lhs | rhs, expected);

        Ok(())
    }

    #[test]
    fn test_field_matcher_bitor_assign() -> TestResult {
        let expected = FieldMatcher::new("012A.a? || 012A.b?")?;
        let mut matcher = FieldMatcher::new("012A.a?")?;
        matcher |= FieldMatcher::new("012A.b?")?;
        assert_eq!(matcher, expected);

        let expected =
            FieldMatcher::new("012A.a? || 012A.b? || 012A.c?")?;
        let mut matcher = FieldMatcher::new("012A.a? || 012A.b?")?;
        matcher |= FieldMatcher::new("012A.c?")?;
        assert_eq!(matcher, expected);

        Ok(())
    }

    #[test]
    fn test_field_matcher_bitnot() -> TestResult {
        let expected = FieldMatcher::new("!(012A.a?)")?;
        let matcher = !FieldMatcher::new("(012A.a?)")?;
        assert_eq!(matcher, expected);

        let expected = FieldMatcher::new("!012A.a?")?;
        let matcher = !FieldMatcher::new("012A.a?")?;
        assert_eq!(matcher, expected);

        let expected = FieldMatcher::new("!!012A.a?")?;
        let matcher = !FieldMatcher::new("!012A.a?")?;
        assert_eq!(matcher, expected);

        let expected = FieldMatcher::new("!(012A.a == 'foo')")?;
        let matcher = !FieldMatcher::new("012A.a == 'foo'")?;
        assert_eq!(matcher, expected);

        let expected = FieldMatcher::new("!(012A.a? && 012A.b?)")?;
        let matcher = !FieldMatcher::new("012A.a? && 012A.b?")?;
        assert_eq!(matcher, expected);

        let expected = FieldMatcher::new("!(012A.a? || 012A.b?)")?;
        let matcher = !FieldMatcher::new("012A.a? || 012A.b?")?;
        assert_eq!(matcher, expected);

        let expected = FieldMatcher::new("!(012A.a? ^ 012A.b?)")?;
        let matcher = !FieldMatcher::new("012A.a? ^ 012A.b?")?;
        assert_eq!(matcher, expected);

        Ok(())
    }
}
