use std::fmt::{self, Display};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not,
};

use winnow::Parser;

use super::field::parser::parse_field_matcher;
use super::field::FieldMatcher;
use super::{MatcherOptions, ParseMatcherError};
use crate::primitives::RecordRef;

/// A matcher that matches against a [RecordRef].
#[derive(Debug, Clone, PartialEq)]
pub struct RecordMatcher(FieldMatcher);

impl RecordMatcher {
    /// Creates a new [RecordMatcher].
    ///
    /// # Errors
    ///
    /// This function fails if the given expression is not a valid
    /// record matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::RecordMatcher;
    ///
    /// let _matcher = RecordMatcher::new("#003@ > 1")?;
    /// let _matcher = RecordMatcher::new("010@.a == 'ger'")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(matcher: &str) -> Result<Self, ParseMatcherError> {
        let matcher = parse_field_matcher
            .parse(matcher.as_bytes())
            .map_err(|_| {
                ParseMatcherError(format!(
                    "invalid field matcher '{matcher}'"
                ))
            })?;

        Ok(Self(matcher))
    }

    /// Returns `true` if the given field(s) matches against the field
    /// matcher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::{MatcherOptions, RecordMatcher};
    /// use pica_record::primitives::RecordRef;
    ///
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// let options = MatcherOptions::default();
    /// let matcher = RecordMatcher::new("002@.0 == 'Tp1'")?;
    /// assert!(matcher.is_match(&record, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    pub fn is_match(
        &self,
        record: &RecordRef,
        options: &MatcherOptions,
    ) -> bool {
        self.0.is_match(record.fields(), options)
    }
}

impl Display for RecordMatcher {
    /// Formats a [RecordMatcher] as a human-readable string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::RecordMatcher;
    ///
    /// let matcher = RecordMatcher::new("002@.0 == 'Tp1'")?;
    /// assert_eq!(matcher.to_string(), "002@.0 == 'Tp1'");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl BitAnd for RecordMatcher {
    type Output = Self;

    /// The bitwise AND operator `&` of two [RecordMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let options = MatcherOptions::default();
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// let lhs = RecordMatcher::new("003@.0 == '123456789X'")?;
    /// let rhs = RecordMatcher::new("002@.0 == 'Tp1'")?;
    /// let matcher = lhs & rhs;
    ///
    /// assert!(matcher.is_match(&record, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for RecordMatcher {
    /// The bitwise AND assignment operator `&=` of two [RecordMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let options = MatcherOptions::default();
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// let mut matcher = RecordMatcher::new("003@.0 == '123456789X'")?;
    /// matcher &= RecordMatcher::new("002@.0 == 'Tp1'")?;
    /// assert!(matcher.is_match(&record, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOr for RecordMatcher {
    type Output = Self;

    /// The bitwise OR operator `|` of two [RecordMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let options = MatcherOptions::default();
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// let lhs = RecordMatcher::new("003@.0 == '234567890X'")?;
    /// let rhs = RecordMatcher::new("002@.0 == 'Tp1'")?;
    /// let matcher = lhs | rhs;
    ///
    /// assert!(matcher.is_match(&record, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for RecordMatcher {
    /// The bitwise OR assignment operator `|=` of two [RecordMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let options = MatcherOptions::default();
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// let mut matcher = RecordMatcher::new("003@.0 == '234567891X'")?;
    /// matcher |= RecordMatcher::new("002@.0 == 'Tp1'")?;
    /// assert!(matcher.is_match(&record, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitXor for RecordMatcher {
    type Output = Self;

    /// The bitwise XOR operator `^` of two [RecordMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let options = MatcherOptions::default();
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// let lhs = RecordMatcher::new("003@.0 == '123456789X'")?;
    /// let rhs = RecordMatcher::new("002@.0 == 'Tp2'")?;
    /// let matcher = lhs ^ rhs;
    ///
    /// assert!(matcher.is_match(&record, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for RecordMatcher {
    /// The bitwise XOR assignment operator `^=` of two [RecordMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let options = MatcherOptions::default();
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// let mut matcher = RecordMatcher::new("003@.0 == '123456789X'")?;
    /// matcher ^= RecordMatcher::new("002@.0 == 'Tp2'")?;
    /// assert!(matcher.is_match(&record, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl Not for RecordMatcher {
    type Output = Self;

    /// The unary logical negation operator `!` applied to a
    /// [RecordMatcher].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::prelude::*;
    /// use pica_record::primitives::RecordRef;
    ///
    /// let options = MatcherOptions::default();
    /// let record = RecordRef::new(vec![
    ///     ("003@", None, vec![('0', "123456789X")]),
    ///     ("002@", None, vec![('0', "Tp1")]),
    /// ])?;
    ///
    /// let matcher = !RecordMatcher::new("002@.0 == 'Tp2'")?;
    /// assert!(matcher.is_match(&record, &options));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RecordMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RecordMatcher {
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
    use std::path::Path;
    use std::sync::OnceLock;
    use std::{env, fs};

    use serde_test::{assert_tokens, Token};

    use super::*;

    type TestResult = anyhow::Result<()>;

    fn ada_lovelace() -> &'static [u8] {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        DATA.get_or_init(|| {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let path =
                Path::new(&manifest_dir).join("tests/data/ada.dat");
            fs::read_to_string(&path).unwrap().as_bytes().to_vec()
        })
    }

    #[test]
    fn test_record_matcher_serde() -> TestResult {
        let matcher =
            RecordMatcher::new("012A.a? && (012A.b? || 012A.c?)")?;

        assert_tokens(
            &matcher,
            &[Token::Str("012A.a? && (012A.b? || 012A.c?)")],
        );

        let matcher =
            RecordMatcher::new("012A.a? && !(012A.b? || 012A.c?) ")?;
        assert_tokens(
            &matcher,
            &[Token::Str("012A.a? && !(012A.b? || 012A.c?)")],
        );

        Ok(())
    }

    #[test]
    fn test_record_matcher_new() -> TestResult {
        let matcher = RecordMatcher::new("003@.0?")?;
        assert!(matcher.is_match(
            &RecordRef::from_bytes(ada_lovelace())?,
            &MatcherOptions::default()
        ));

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_record_matcher_new_panic() {
        let _ = RecordMatcher::new("003@.!?").unwrap();
    }

    #[test]
    fn test_record_matcher_exists() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;

        let matcher = RecordMatcher::new("004B?")?;
        assert!(matcher.is_match(&record, &Default::default()));

        let matcher = RecordMatcher::new("028A.a?")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_cardinality() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher = RecordMatcher::new(
            "#028[A@]{d =^ 'Ada' && a == 'Lovelace'} == 5",
        )?;

        assert!(matcher.is_match(&record, &Default::default()));
        Ok(())
    }

    #[test]
    fn test_record_matcher_in() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher = RecordMatcher::new("002@.0 in ['Tpz', 'Tp1']")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_not_in() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher =
            RecordMatcher::new("002@.0 not in ['Tuz', 'Tu1']")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_regex() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher =
            RecordMatcher::new("047A/03.[er] =~ '^DE-\\\\d+6'")?;

        assert!(matcher.is_match(&record, &Default::default()));

        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher =
            RecordMatcher::new("047A/03.[er] !~ '^EN-\\\\d+6'")?;

        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_regex_set() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher = RecordMatcher::new(
            "047A/03.[er] =~ ['^DE-\\\\d+6', '^EN-.*']",
        )?;

        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_equal() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher = RecordMatcher::new("003@.0 == '119232022'")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_not_equal() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher = RecordMatcher::new("002@.0 != 'Ts1'")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_starts_with() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher =
            RecordMatcher::new("003U.a =^ 'http://d-nb.info/gnd/'")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_starts_not_with() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher = RecordMatcher::new("002@.0 !^ 'Ts'")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_ends_with() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher = RecordMatcher::new("042B.a =$ '-GB'")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_ends_not_with() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher = RecordMatcher::new("002@.0 !$ '3'")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_group() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher =
            RecordMatcher::new("(002@.0 == 'Tp1' && 004B.a == 'pik')")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_not() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher =
            RecordMatcher::new("!(002@.0 == 'Ts1' || 002@.0 =^ 'Tu')")?;
        assert!(matcher.is_match(&record, &Default::default()));

        let matcher = RecordMatcher::new("!012A.0?")?;
        assert!(matcher.is_match(&record, &Default::default()));
        Ok(())
    }

    #[test]
    fn test_record_matcher_and() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher =
            RecordMatcher::new("002@.0 == 'Tp1' && 004B.a == 'pik'")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_or() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;
        let matcher =
            RecordMatcher::new("002@.0 == 'Ts1' || 004B.a == 'pik'")?;
        assert!(matcher.is_match(&record, &Default::default()));

        Ok(())
    }

    #[test]
    fn test_record_matcher_xor() -> TestResult {
        let record = RecordRef::from_bytes(ada_lovelace())?;

        let matcher =
            RecordMatcher::new("042A.a == '28p' ^ 042A.a == '9.5p'")?;
        assert!(!matcher.is_match(&record, &Default::default()));

        let matcher =
            RecordMatcher::new("042A.a == '28p' XOR 042A.a == '9.5p'")?;
        assert!(!matcher.is_match(&record, &Default::default()));

        Ok(())
    }
}
