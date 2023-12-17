use std::str::FromStr;

use bstr::B;
use pica_matcher::subfield_matcher::*;
use pica_matcher::{
    MatcherOptions, ParseMatcherError, Quantifier, RelationalOp,
};
use pica_record::SubfieldRef;

use crate::TestResult;

macro_rules! subfield {
    ($code:expr, $value:expr) => {
        SubfieldRef::new($code, $value)
    };
}

#[test]
fn exists_matcher_new() {
    let subfield = subfield!('0', "119232022");
    let options = MatcherOptions::default();

    let matcher = ExistsMatcher::new(vec!['0']);
    assert!(matcher.is_match(&subfield, &options));

    let matcher = ExistsMatcher::new(vec!['2', '3']);
    assert!(!matcher.is_match(&subfield, &options));
}

#[test]
#[should_panic]
fn exists_matcher_new_panic() {
    let _ = ExistsMatcher::new(vec!['0', '!']);
}

#[test]
fn exists_matcher_try_from() -> TestResult {
    let subfield = subfield!('0', "119232022");
    let options = MatcherOptions::default();

    let matcher = ExistsMatcher::try_from(B("0?"))?;
    assert!(matcher.is_match(&subfield, &options));

    assert!(matches!(
        ExistsMatcher::try_from("ä?".as_bytes()).unwrap_err(),
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn exists_matcher_from_str() -> TestResult {
    let subfield = subfield!('0', "119232022");
    let options = MatcherOptions::default();

    let matcher = ExistsMatcher::from_str("0?")?;
    assert!(matcher.is_match(&subfield, &options));

    assert!(matches!(
        ExistsMatcher::from_str("ä?").unwrap_err(),
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn exists_matcher_is_match() -> TestResult {
    let matcher = ExistsMatcher::from_str("1?")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('1', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));

    assert!(matcher.is_match(
        [&subfield!('3', "def"), &subfield!('1', "hij"),],
        &options
    ));

    let matcher = ExistsMatcher::from_str("[a12]?")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('1', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('9', "abc"), &options));
    assert!(matcher.is_match(
        [
            &subfield!('3', "def"),
            &subfield!('9', "hij"),
            &subfield!('2', "bsg"),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn relational_matcher_new() {
    let matcher = RelationMatcher::new("0 == 'abc'");
    assert!(matcher
        .is_match(&subfield!('0', "abc"), &MatcherOptions::new()));
}

#[test]
#[should_panic]
fn relational_matcher_new_panic() {
    let _ = RelationMatcher::new("! == 'abc'");
}

#[test]
fn relation_matcher_try_from() -> TestResult {
    let matcher = RelationMatcher::try_from(B("0 == 'abc'"))?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));

    Ok(())
}

#[test]
fn relation_matcher_from_str() -> TestResult {
    let matcher = RelationMatcher::from_str("0 == 'abc'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));

    Ok(())
}

#[test]
fn relational_matcher_equal() {
    // case sensitive
    let matcher = RelationMatcher::new("0 == 'abc'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "ABC"), &options));
    assert!(!matcher.is_match(&subfield!('1', "abc"), &options));
    assert!(matcher.is_match(
        [
            &subfield!('3', "def"),
            &subfield!('0', "abc"),
            &subfield!('2', "bsg"),
        ],
        &options
    ));

    // case insensitive
    let matcher = RelationMatcher::new("0 == 'abc'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "ABC"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 == 'abc'");
    let options = MatcherOptions::default();

    let subfields = [
        &subfield!('3', "def"),
        &subfield!('0', "abc"),
        &subfield!('2', "hij"),
    ];

    assert!(matcher.is_match(subfields, &options));

    let subfields = [&subfield!('3', "def"), &subfield!('2', "hij")];
    assert!(!matcher.is_match(subfields, &options));
}

#[test]
fn relational_matcher_not_equal() {
    // case sensitive
    let matcher = RelationMatcher::new("0 != 'abc'");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "ABC"), &options));
    assert!(!matcher.is_match(&subfield!('1', "abc"), &options));

    let subfields = [&subfield!('0', "abc"), &subfield!('2', "hij")];
    assert!(!matcher.is_match(subfields, &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 != 'abc'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "ABC"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 != 'abc'");
    let options = MatcherOptions::default();

    let subfields = [&subfield!('3', "def"), &subfield!('0', "bsg")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [
        &subfield!('3', "def"),
        &subfield!('0', "abc"),
        &subfield!('2', "bsg"),
    ];

    assert!(!matcher.is_match(subfields, &options));
}

#[test]
fn relational_matcher_starts_not_with() {
    // case sensitive
    let matcher = RelationMatcher::new("0 !^ 'ab'");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "def"), &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 !^ 'ab'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "ABc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "aBc"), &options));
    assert!(matcher.is_match(&subfield!('0', "def"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 !^ 'ab'");
    let options = MatcherOptions::default();

    let subfields = [&subfield!('0', "baab"), &subfield!('0', "abba")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [&subfield!('0', "abc"), &subfield!('1', "abba")];
    assert!(!matcher.is_match(subfields, &options));
}

#[test]
fn relational_matcher_ends_with() {
    // case sensitive
    let matcher = RelationMatcher::new("0 =$ 'ab'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abab"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abba"), &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 =$ 'ab'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('0', "abab"), &options));
    assert!(matcher.is_match(&subfield!('0', "abab"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 =$ 'ab'");
    let options = MatcherOptions::default();

    let subfields = [&subfield!('0', "baba"), &subfield!('0', "abab")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [&subfield!('0', "def"), &subfield!('1', "aab")];
    assert!(!matcher.is_match(subfields, &options));
}

#[test]
fn relational_matcher_ends_not_with() {
    // case sensitive
    let matcher = RelationMatcher::new("0 !$ 'ab'");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('0', "abab"), &options));
    assert!(matcher.is_match(&subfield!('0', "abba"), &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 !$ 'ab'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(!matcher.is_match(&subfield!('0', "abab"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abAB"), &options));
    assert!(matcher.is_match(&subfield!('0', "abbba"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 !$ 'ab'");
    let options = MatcherOptions::default();

    let subfields = [&subfield!('0', "baba"), &subfield!('0', "abab")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [&subfield!('0', "abab"), &subfield!('1', "ab")];
    assert!(!matcher.is_match(subfields, &options));
}

#[test]
fn relational_matcher_similar() {
    // default threshold
    let matcher = RelationMatcher::new("a =* 'Heike'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "Heike"), &options));
    assert!(!matcher.is_match(&subfield!('a', "Heiko"), &options));

    // threshold set
    let matcher = RelationMatcher::new("a =* 'Heike'");
    let options = MatcherOptions::new().strsim_threshold(0.7);

    assert!(matcher.is_match(&subfield!('a', "Heike"), &options));
    assert!(matcher.is_match(&subfield!('a', "Heiko"), &options));

    // default threshold
    let matcher = RelationMatcher::new("a =* 'Heike'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('a', "Heike"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("a =* 'Heike'");
    let options = MatcherOptions::default();

    let subfields =
        [&subfield!('a', "Heiko"), &subfield!('a', "Heike")];
    assert!(matcher.is_match(subfields, &options));
}

#[test]
fn relational_matcher_contains() {
    // default options
    let matcher = RelationMatcher::new("a =? 'aba'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "aba"), &options));
    assert!(matcher.is_match(&subfield!('a', "xabax"), &options));
    assert!(!matcher.is_match(&subfield!('a', "abba"), &options));

    // case ignore
    let matcher = RelationMatcher::new("a =? 'AbA'");
    let options = MatcherOptions::default().case_ignore(true);

    assert!(matcher.is_match(&subfield!('a', "aba"), &options));
    assert!(matcher.is_match(&subfield!('a', "xabax"), &options));
    assert!(!matcher.is_match(&subfield!('a', "abba"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("a =? 'aba'");
    let options = MatcherOptions::default();

    let subfields =
        [&subfield!('a', "XabbaX"), &subfield!('a', "YabaY")];
    assert!(matcher.is_match(subfields, &options));
}

#[test]
fn regex_matcher_new() {
    let _ = RegexMatcher::new(
        vec!['0'],
        "^T[gpsu][1z]$",
        Quantifier::Forall,
        false,
    );
}

#[test]
#[should_panic]
fn regex_matcher_new_panic1() {
    RegexMatcher::new(
        vec!['0'],
        "^T[[gpsu][1z]$",
        Quantifier::Exists,
        false,
    );
}

#[test]
fn regex_matcher_try_from() {
    assert!(RegexMatcher::try_from(B("0 =~ '^T[gpsu][1z]$'")).is_ok());

    let error =
        RegexMatcher::try_from(B("0 =~ '^Tp[[1z]$'")).unwrap_err();
    assert!(matches!(
        error,
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));
}

#[test]
fn regex_matcher_from_str() {
    assert!(RegexMatcher::from_str("0 =~ '^T[gpsu][1z]$'").is_ok());

    let error = RegexMatcher::from_str("0 =~ '^Tp[[1z]$'").unwrap_err();

    assert!(matches!(
        error,
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));
}

#[test]
fn regex_matcher_is_match() -> TestResult {
    // case sensitive
    let matcher = RegexMatcher::from_str("0 =~ '^ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abba"), &options));
    assert!(!matcher.is_match(&subfield!('0', "bba"), &options));
    assert!(!matcher.is_match(&subfield!('a', "abba"), &options));

    // case insensitive
    let matcher = RegexMatcher::from_str("0 =~ '^ab'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('0', "abba"), &options));
    assert!(matcher.is_match(&subfield!('0', "abba"), &options));

    // invert match
    let matcher = RegexMatcher::from_str("0 !~ '^ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "baba"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abba"), &options));

    // multiple subfields
    let matcher = RegexMatcher::from_str("0 =~ '^ab'")?;
    let options = MatcherOptions::default();

    let subfields =
        [&subfield!('0', "foobar"), &subfield!('0', "abba")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [&subfield!('0', "foo"), &subfield!('0', "bar")];
    assert!(!matcher.is_match(subfields, &options));

    Ok(())
}

#[test]
fn in_matcher_new() {
    assert!(InMatcher::new(
        vec!['0'],
        vec!["abc", "def"],
        Quantifier::Forall,
        false
    )
    .is_match(&subfield!('0', "abc"), &MatcherOptions::default()));
}

#[test]
#[should_panic]
fn in_matcher_new_panic() {
    let _ = InMatcher::new(
        vec!['!'],
        vec!["abc", "def"],
        Quantifier::Exists,
        false,
    );
}

#[test]
fn in_matcher_try_from() -> TestResult {
    let matcher = InMatcher::try_from(B("0 in ['abc', 'def']"))?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));

    Ok(())
}

#[test]
fn in_matcher_from_str() -> TestResult {
    let matcher = InMatcher::from_str("0 in ['abc', 'def']")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));

    Ok(())
}

#[test]
fn in_matcher_is_match() -> TestResult {
    // case sensitive
    let matcher = InMatcher::from_str("0 in ['abc', 'def']")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "ABC"), &options));
    assert!(matcher.is_match(&subfield!('0', "def"), &options));
    assert!(!matcher.is_match(&subfield!('0', "DEF"), &options));
    assert!(!matcher.is_match(&subfield!('0', "hij"), &options));

    // case insensitive
    let matcher = InMatcher::from_str("0 in ['abc', 'def']")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "ABC"), &options));

    // multiple subfields
    let matcher = InMatcher::from_str("0 in ['abc', 'def']")?;
    let options = MatcherOptions::default();

    let subfields = [&subfield!('0', "hij"), &subfield!('0', "abc")];
    assert!(matcher.is_match(subfields, &options));

    let matcher = InMatcher::from_str("a in ['000', '999']")?;
    let options = MatcherOptions::default();

    let subfields = [&subfield!('a', "000"), &subfield!('z', "xyz")];
    assert!(matcher.is_match(subfields, &options));

    // invert
    let matcher = InMatcher::from_str("a not in ['000', '999']")?;
    let options = MatcherOptions::default();

    let subfields = [&subfield!('a', "000"), &subfield!('a', "222")];
    assert!(matcher.is_match(subfields, &options));

    let matcher = InMatcher::from_str("a not in ['000', '999']")?;
    let options = MatcherOptions::default();

    let subfields = [&subfield!('a', "000"), &subfield!('z', "xyz")];
    assert!(!matcher.is_match(subfields, &options));

    Ok(())
}

#[test]
fn cardinality_matcher_new() {
    let matcher = CardinalityMatcher::new('0', RelationalOp::Eq, 2);
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('X', "abc"), &options));
}

#[test]
#[should_panic]
fn cardinality_matcher_new_panic1() {
    let _ = CardinalityMatcher::new('!', RelationalOp::Eq, 2);
}

#[test]
#[should_panic]
fn cardinality_matcher_new_panic2() {
    let _ = CardinalityMatcher::new('!', RelationalOp::StartsWith, 2);
}

#[test]
fn cardinality_matcher_try_from() -> TestResult {
    let matcher = CardinalityMatcher::try_from(B("#0 == 2"))?;
    assert!(!matcher
        .is_match(&subfield!('X', "abc"), &MatcherOptions::default()));

    assert!(matches!(
        CardinalityMatcher::try_from(B("#0 =~ 2")).unwrap_err(),
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_try_str() -> TestResult {
    let matcher = CardinalityMatcher::from_str("#0 == 2")?;
    assert!(!matcher
        .is_match(&subfield!('X', "abc"), &MatcherOptions::default()));

    assert!(matches!(
        CardinalityMatcher::from_str("#0 =~ 2").unwrap_err(),
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_equal() -> TestResult {
    let matcher = CardinalityMatcher::from_str("#0 == 2")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('X', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));

    let subfields = [&subfield!('0', "abc"), &subfield!('0', "def")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [
        &subfield!('0', "abc"),
        &subfield!('0', "def"),
        &subfield!('0', "hij"),
    ];
    assert!(!matcher.is_match(subfields, &options));

    Ok(())
}

#[test]
fn cardinality_matcher_not_equal() -> TestResult {
    let matcher = CardinalityMatcher::from_str("#0 != 2")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('X', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "abc"), &options));

    let subfields = [&subfield!('0', "abc"), &subfield!('0', "def")];
    assert!(!matcher.is_match(subfields, &options));

    let subfields = [
        &subfield!('0', "abc"),
        &subfield!('0', "def"),
        &subfield!('0', "hij"),
    ];
    assert!(matcher.is_match(subfields, &options));

    Ok(())
}

#[test]
fn cardinality_matcher_greater_than_or_equal() -> TestResult {
    let matcher = CardinalityMatcher::from_str("#0 >= 2")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('X', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));

    let subfields = [&subfield!('0', "abc"), &subfield!('0', "def")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [
        &subfield!('0', "abc"),
        &subfield!('0', "def"),
        &subfield!('0', "hij"),
    ];
    assert!(matcher.is_match(subfields, &options));

    Ok(())
}

#[test]
fn cardinality_matcher_greater_than() -> TestResult {
    let matcher = CardinalityMatcher::from_str("#0 > 2")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('X', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));

    let subfields = [&subfield!('0', "abc"), &subfield!('0', "def")];
    assert!(!matcher.is_match(subfields, &options));

    let subfields = [
        &subfield!('0', "abc"),
        &subfield!('0', "def"),
        &subfield!('0', "hij"),
    ];
    assert!(matcher.is_match(subfields, &options));

    Ok(())
}

#[test]
fn cardinality_matcher_less_than_or_equal() -> TestResult {
    let matcher = CardinalityMatcher::from_str("#0 <= 2")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('X', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "abc"), &options));

    let subfields = [&subfield!('0', "abc"), &subfield!('0', "def")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [
        &subfield!('0', "abc"),
        &subfield!('0', "def"),
        &subfield!('0', "hij"),
    ];
    assert!(!matcher.is_match(subfields, &options));

    Ok(())
}

#[test]
fn cardinality_matcher_less_than() -> TestResult {
    let matcher = CardinalityMatcher::from_str("#0 < 2")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('X', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "abc"), &options));

    let subfields = [&subfield!('0', "abc"), &subfield!('0', "def")];
    assert!(!matcher.is_match(subfields, &options));

    let subfields = [
        &subfield!('0', "abc"),
        &subfield!('0', "def"),
        &subfield!('0', "hij"),
    ];
    assert!(!matcher.is_match(subfields, &options));
    Ok(())
}

#[test]
fn subfield_matcher_new() {
    let matcher = SubfieldMatcher::new("a == 'bcd'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "bcd"), &options));
}

#[test]
#[should_panic]
fn subfield_matcher_new_panic() {
    let _ = SubfieldMatcher::new("a == 'bcd");
}

#[test]
fn subfield_matcher_try_from() -> TestResult {
    let matcher = SubfieldMatcher::try_from(B("a == 'bcd'"))?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "bcd"), &options));

    assert!(matches!(
        SubfieldMatcher::try_from(B("a == 'bcd")).unwrap_err(),
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn subfield_matcher_from_str() -> TestResult {
    let matcher = SubfieldMatcher::from_str("a == 'bcd'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "bcd"), &options));

    assert!(matches!(
        SubfieldMatcher::from_str("a == 'bcd").unwrap_err(),
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn subfield_matcher_bit_and() -> TestResult {
    let lhs = SubfieldMatcher::from_str("a =^ 'D'")?;
    let rhs = SubfieldMatcher::from_str("a =$ 'NB'")?;
    let matcher = lhs & rhs;

    assert!(matcher
        .is_match(&subfield!('a', "DNB"), &MatcherOptions::default()));

    Ok(())
}

#[test]
fn subfield_matcher_bit_or() -> TestResult {
    let lhs = SubfieldMatcher::from_str("a =^ 'f'")?;
    let rhs = SubfieldMatcher::from_str("a =^ 'b'")?;
    let matcher = lhs | rhs;

    assert!(matcher
        .is_match(&subfield!('a', "foo"), &MatcherOptions::default()));
    assert!(matcher
        .is_match(&subfield!('a', "bar"), &MatcherOptions::default()));

    Ok(())
}

#[test]
fn subfield_matcher_not() -> TestResult {
    // group
    let matcher = SubfieldMatcher::from_str("!(a == 'bcd')")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('a', "bcd"), &options));
    assert!(matcher.is_match(&subfield!('b', "cde"), &options));

    // exists
    let matcher = SubfieldMatcher::new("!a?");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('a', "bcd"), &options));
    assert!(matcher.is_match(&subfield!('b', "cde"), &options));

    // not
    let matcher = SubfieldMatcher::new("!!!(a == 'bcd')");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('a', "bcd"), &options));
    assert!(matcher.is_match(&subfield!('b', "cde"), &options));

    Ok(())
}

#[test]
fn subfield_matcher_group() {
    // and
    let matcher = SubfieldMatcher::new("(a =^ 'ab' && a =$ 'ba')");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "abba"), &options));
    assert!(!matcher.is_match(&subfield!('b', "cde"), &options));

    // or
    let matcher = SubfieldMatcher::new("(a =^ 'ab' || a =^ 'ba')");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "abba"), &options));
    assert!(matcher.is_match(&subfield!('a', "baba"), &options));

    // singleton
    let matcher = SubfieldMatcher::new("(a == 'bcd')");
    let options = MatcherOptions::default();
    assert!(matcher.is_match(&subfield!('a', "bcd"), &options));
    assert!(!matcher.is_match(&subfield!('b', "cde"), &options));

    // nested group
    let matcher = SubfieldMatcher::new("(((a == 'bcd')))");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "bcd"), &options));

    // not
    let matcher = SubfieldMatcher::new("(!(a == 'bcd'))");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('h', "ijk"), &options));
}

#[test]
fn subfield_matcher_or() -> TestResult {
    // singleton
    let matcher =
        SubfieldMatcher::new("a =^ 'ab' || a =^ 'bc' || a =^ 'cd'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "abab"), &options));
    assert!(matcher.is_match(&subfield!('a', "bcbc"), &options));
    assert!(matcher.is_match(&subfield!('a', "cdcd"), &options));
    assert!(!matcher.is_match(&subfield!('a', "dede"), &options));
    assert!(!matcher.is_match(&subfield!('b', "abab"), &options));

    // group
    let matcher =
        SubfieldMatcher::new("a =^ 'ab' || (a =^ 'bc' && a =$ 'cd')");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "abab"), &options));
    assert!(matcher.is_match(&subfield!('a', "bccd"), &options));
    assert!(!matcher.is_match(&subfield!('a', "bcbc"), &options));

    // and
    let matcher =
        SubfieldMatcher::new("a =^ 'ab' || a =^ 'bc' && a =$ 'cd'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "abab"), &options));
    assert!(matcher.is_match(&subfield!('a', "abcd"), &options));
    assert!(matcher.is_match(&subfield!('a', "bccd"), &options));
    assert!(!matcher.is_match(&subfield!('a', "bcbc"), &options));

    // or
    let matcher = SubfieldMatcher::new("!a? || b == 'x'");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('a', "abab"), &options));

    let subfields = [&subfield!('a', "bccd"), &subfield!('b', "x")];
    assert!(matcher.is_match(subfields, &options));

    // not
    let matcher = SubfieldMatcher::new("a == 'bcd' || !(a != 'def')");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "bcd"), &options));
    assert!(matcher.is_match(&subfield!('a', "def"), &options));
    assert!(!matcher.is_match(&subfield!('a', "hij"), &options));

    // boolean op precedence
    let matcher =
        SubfieldMatcher::new("(a =^ 'ab' || a =^ 'bc') && a =$ 'cd'");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('a', "abab"), &options));
    assert!(matcher.is_match(&subfield!('a', "abcd"), &options));
    assert!(matcher.is_match(&subfield!('a', "bccd"), &options));
    assert!(!matcher.is_match(&subfield!('a', "bcbc"), &options));

    // multiple subfields
    let matcher = SubfieldMatcher::new("#a == 2 || a =^ 'ab'");
    let options = MatcherOptions::default();

    let subfields = [&subfield!('a', "def"), &subfield!('a', "hij")];
    assert!(matcher.is_match(subfields, &options));

    let subfields = [
        &subfield!('a', "def"),
        &subfield!('a', "hij"),
        &subfield!('a', "abc"),
    ];
    assert!(matcher.is_match(subfields, &options));

    Ok(())
}

#[test]
fn subfield_matcher_and() -> anyhow::Result<()> {
    // singleton
    let matcher =
        SubfieldMatcher::new("#a == 1 && a =^ 'ab' && a =$ 'ba'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "abba"), &options));
    assert!(!matcher.is_match(&subfield!('a', "baba"), &options));

    let subfields = [&subfield!('a', "abba"), &subfield!('a', "baba")];
    assert!(!matcher.is_match(subfields, &options));

    // group
    let matcher =
        SubfieldMatcher::new("#a == 1 && (a =^ 'ab' || a =^ 'ba')");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "abba"), &options));
    assert!(matcher.is_match(&subfield!('a', "baba"), &options));

    let subfields = [&subfield!('a', "abba"), &subfield!('a', "baba")];
    assert!(!matcher.is_match(subfields, &options));

    // not
    let matcher =
        SubfieldMatcher::new("#a == 1 && !(a =^ 'ab' || a =^ 'ba')");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('a', "abba"), &options));
    assert!(!matcher.is_match(&subfield!('a', "baba"), &options));
    assert!(matcher.is_match(&subfield!('a', "cbcb"), &options));

    Ok(())
}
