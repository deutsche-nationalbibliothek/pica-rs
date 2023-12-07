use std::str::FromStr;

use bstr::B;
use pica_matcher::field_matcher::{
    CardinalityMatcher, ExistsMatcher, SingletonMatcher,
    SubfieldsMatcher,
};
use pica_matcher::{FieldMatcher, MatcherOptions, ParseMatcherError};
use pica_record::FieldRef;

use crate::TestResult;

macro_rules! field {
    ($tag:expr, $code:expr, $value:expr) => {
        FieldRef::new($tag, None, vec![($code, $value)])
    };

    ($tag:expr, $occurrence:expr, $code:expr, $value:expr) => {
        FieldRef::new($tag, Some($occurrence), vec![($code, $value)])
    };
}

#[test]
fn exists_matcher_new() {
    let matcher = ExistsMatcher::new("003@?");
    let options = MatcherOptions::default();

    assert!(
        matcher.is_match(&field!("003@", '0', "123456789X"), &options)
    );
}

#[test]
#[should_panic]
fn exists_matcher_new_panic() {
    let _ = ExistsMatcher::new("303@?");
}

#[test]
fn exists_matcher_try_from() -> TestResult {
    let matcher = ExistsMatcher::try_from(B("003@?"))?;
    let options = MatcherOptions::default();

    assert!(
        matcher.is_match(&field!("003@", '0', "123456789X"), &options)
    );

    assert!(matches!(
        ExistsMatcher::try_from(B("303@?")).unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn exists_matcher_from_str() -> TestResult {
    let matcher = ExistsMatcher::from_str("003@?")?;
    let options = MatcherOptions::default();

    assert!(
        matcher.is_match(&field!("003@", '0', "123456789X"), &options)
    );

    assert!(matches!(
        ExistsMatcher::from_str("303@?").unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn exists_matcher_is_match() -> TestResult {
    let matcher = ExistsMatcher::new("003@?");
    let options = MatcherOptions::default();

    assert!(
        matcher.is_match(&field!("003@", '0', "123456789X"), &options)
    );

    assert!(!matcher.is_match(&field!("002@", '0', "Olfo"), &options));

    let fields = [
        &field!("002@", '0', "Olfo"),
        &field!("003@", '0', "123456789X"),
    ];
    assert!(matcher.is_match(fields, &options));

    let matcher = ExistsMatcher::new("00[23]@?");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("002@", '0', "Olfo"), &options));
    assert!(
        matcher.is_match(&field!("003@", '0', "123456789X"), &options)
    );

    // occurrence
    let matcher = ExistsMatcher::new("041A/02?");
    let options = MatcherOptions::default();

    let field = field!("041A", "01", 'a', "abc");
    assert!(!matcher.is_match(&field, &options));

    let field = field!("041A", "02", 'a', "abc");
    assert!(matcher.is_match(&field, &options));

    Ok(())
}

#[test]
fn subfields_matcher_new() {
    let matcher = SubfieldsMatcher::new("003@.0 == '123456789X'");
    let options = MatcherOptions::default();

    assert!(
        matcher.is_match(&field!("003@", '0', "123456789X"), &options)
    );
}

#[test]
#[should_panic]
fn subfields_matcher_new_panic() {
    let _ = SubfieldsMatcher::new("003!.0 == '123456789X'");
}

#[test]
fn subfields_matcher_try_from() -> TestResult {
    let matcher =
        SubfieldsMatcher::try_from(B("003@.0 == '123456789X'"))?;
    let options = MatcherOptions::default();

    assert!(
        matcher.is_match(&field!("003@", '0', "123456789X"), &options)
    );

    assert!(matches!(
        SubfieldsMatcher::try_from(B("003@.! == '123456789X'"))
            .unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn subfields_matcher_from_str() -> TestResult {
    let matcher = SubfieldsMatcher::from_str("003@.0 == '123456789X'")?;
    let options = MatcherOptions::default();

    assert!(
        matcher.is_match(&field!("003@", '0', "123456789X"), &options)
    );

    assert!(matches!(
        SubfieldsMatcher::from_str("003@.! == '123456789X'")
            .unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn subfields_matcher_is_match() -> TestResult {
    // simple
    let matcher = SubfieldsMatcher::new("003@.0 == '123456789X'");
    let options = MatcherOptions::default();

    let field = field!("003@", '0', "123456789X");
    assert!(matcher.is_match(&field, &options));

    let field = field!("002@", '0', "Olfo");
    assert!(!matcher.is_match(&field, &options));

    // complex
    let matcher =
        SubfieldsMatcher::new("003@{0? && 0 == '123456789X'}");
    let options = MatcherOptions::default();

    let field = field!("003@", '0', "123456789X");
    assert!(matcher.is_match(&field, &options));

    let field = field!("003@", '0', "34567");
    assert!(!matcher.is_match(&field, &options));

    let field = field!("002@", '0', "Olfo");
    assert!(!matcher.is_match(&field, &options));

    Ok(())
}

#[test]
fn singleton_matcher_new() {
    let matcher = SingletonMatcher::new("041A/03.9?");
    let options = MatcherOptions::default();
    let field = field!("041A", "03", '9', "1234");
    assert!(matcher.is_match(&field, &options));
}

#[test]
fn singleton_matcher_try_from() -> TestResult {
    let matcher = SingletonMatcher::try_from(B("041A/03.9?"))?;
    let options = MatcherOptions::default();
    let field = field!("041A", "03", '9', "1234");
    assert!(matcher.is_match(&field, &options));

    assert!(matches!(
        SingletonMatcher::try_from(B("041A/03.!?")).unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn singleton_matcher_from_str() -> TestResult {
    let matcher = SingletonMatcher::from_str("041A/03.9?")?;
    let options = MatcherOptions::default();
    let field = field!("041A", "03", '9', "1234");
    assert!(matcher.is_match(&field, &options));

    assert!(matches!(
        SingletonMatcher::from_str("041A/03.!?").unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
#[should_panic]
fn singleton_matcher_new_panic() {
    let _ = SingletonMatcher::new("041!/*?");
}

#[test]
fn singleton_matcher_is_match() {
    let matcher = SingletonMatcher::new("041A/*?");
    let options = MatcherOptions::default();
    let field = field!("041A", "09", '9', "1234");
    assert!(matcher.is_match(&field, &options));

    let matcher = SingletonMatcher::new("041A/*.9 == '1234'");
    let options = MatcherOptions::default();
    let field = field!("041A", "09", '9', "1234");
    assert!(matcher.is_match(&field, &options));
}

#[test]
fn cardinality_matcher_new() {
    let matcher = CardinalityMatcher::new("#012A == 1");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
}

#[test]
#[should_panic]
fn cardinality_matcher_new_panic() {
    let _ = CardinalityMatcher::new("#012A == -1");
}

#[test]
fn cardinality_matcher_try_from() -> TestResult {
    let matcher = CardinalityMatcher::try_from(B("#012A == 1"))?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));

    assert!(matches!(
        CardinalityMatcher::try_from(B("#012A == -1")).unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_from_str() -> TestResult {
    let matcher = CardinalityMatcher::from_str("#012A == 1")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));

    assert!(matches!(
        CardinalityMatcher::from_str("#012A == -1").unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_equal() -> TestResult {
    let matcher = CardinalityMatcher::new("#012A == 1");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(!matcher.is_match(
        vec![&field!("012A", '0', "abc"), &field!("012A", '0', "def"),],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_not_equal() -> TestResult {
    let matcher = CardinalityMatcher::new("#012A{0 =^ 'ab'} != 1");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(
        vec![&field!("012A", '0', "abc"), &field!("012A", '0', "abd")],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_greater_than_or_equal() -> TestResult {
    let matcher = CardinalityMatcher::new("#012A >= 2");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(
        vec![&field!("012A", '0', "abc"), &field!("012A", '0', "def")],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_greater_than() -> TestResult {
    let matcher = CardinalityMatcher::new("#012A{ 0 =^ 'ab' } > 1");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(!matcher.is_match(
        vec![&field!("012A", '0', "abc"), &field!("012A", '0', "def")],
        &options
    ));

    assert!(matcher.is_match(
        vec![
            &field!("012A", '0', "abc"),
            &field!("012A", 'X', "def"),
            &field!("012A", '0', "abd"),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_less_than_or_equal() -> TestResult {
    let matcher = CardinalityMatcher::new("#012A <= 2");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(
        vec![&field!("012A", '0', "abc"), &field!("012A", '0', "def")],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &field!("012A", '0', "abc"),
            &field!("012A", '0', "def"),
            &field!("012A", '0', "hij"),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_less_than() -> TestResult {
    let matcher = CardinalityMatcher::new("#012A{ 0 =^ 'ab' } < 2");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(
        vec![&field!("012A", '0', "abc"), &field!("012A", '0', "def")],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &field!("012A", '0', "abc"),
            &field!("012A", 'X', "def"),
            &field!("012A", '0', "abd")
        ],
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_new() {
    let matcher = FieldMatcher::new("003@.0?");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("003@", '0', "abc"), &options));
}

#[test]
#[should_panic]
fn field_matcher_new_panic() {
    let _ = FieldMatcher::new("003@.!?");
}

#[test]
fn field_matcher_try_from() -> TestResult {
    let matcher = FieldMatcher::try_from(B("003@.0?"))?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("003@", '0', "abc"), &options));
    assert!(matches!(
        FieldMatcher::try_from(B("003@.!?")).unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn field_matcher_from_str() -> TestResult {
    let matcher = FieldMatcher::from_str("003@.0?")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("003@", '0', "abc"), &options));
    assert!(matches!(
        FieldMatcher::from_str("003@.!?").unwrap_err(),
        ParseMatcherError::InvalidFieldMatcher(_)
    ));

    Ok(())
}

#[test]
fn field_matcher_bit_and() {
    let lhs = FieldMatcher::new("044H.9?");
    let rhs = FieldMatcher::new("044H.b == 'gnd'");
    let matcher = lhs & rhs;

    let field =
        FieldRef::new("044H", None, vec![('9', "123"), ('b', "gnd")]);
    assert!(matcher.is_match(&field, &MatcherOptions::default()));
}

#[test]
fn field_matcher_bit_or() {
    let lhs = FieldMatcher::new("044H.9?");
    let rhs = FieldMatcher::new("044K.9?");
    let matcher = lhs | rhs;

    let field =
        FieldRef::new("044K", None, vec![('9', "123"), ('b', "kasw")]);
    assert!(matcher.is_match(&field, &MatcherOptions::default()));
}

#[test]
fn field_matcher_negate() {
    let inner = FieldMatcher::new("044H.9?");
    let matcher = !inner;

    let field =
        FieldRef::new("044K", None, vec![('9', "123"), ('b', "gnd")]);
    assert!(matcher.is_match(&field, &MatcherOptions::default()));
}

#[test]
fn field_matcher_singleton() {
    let matcher = FieldMatcher::new("041A/03.9?");
    assert!(matcher.is_match(
        &field!("041A", "03", '9', "1234"),
        &MatcherOptions::default()
    ));
}

#[test]
fn field_matcher_cardinality() {
    let matcher = FieldMatcher::new("#012A == 1");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
}

#[test]
fn field_matcher_group() {
    // singleton
    let matcher = FieldMatcher::new("(012A?)");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(!matcher.is_match(&field!("013A", '0', "abc"), &options));

    // not
    let matcher = FieldMatcher::new("(!012A?)");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(&field!("013A", '0', "abc"), &options));

    // cardinality
    let matcher = FieldMatcher::new("(#012A <= 1)");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(&field!("013A", '0', "abc"), &options));

    // group
    let matcher = FieldMatcher::new("((012A?))");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(!matcher.is_match(&field!("013A", '0', "abc"), &options));

    // and
    let matcher = FieldMatcher::new("(012A? && 012A.0 == 'abc')");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(!matcher.is_match(&field!("012A", '0', "def"), &options));

    // or
    let matcher = FieldMatcher::new("(012A? || 013A.0 == 'abc')");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(&field!("013A", '0', "abc"), &options));
    assert!(!matcher.is_match(&field!("013A", '0', "def"), &options));
}

#[test]
fn field_matcher_not() {
    // Group
    let matcher = FieldMatcher::new("!(012A?)");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(&field!("013A", '0', "abc"), &options));

    // exists
    let matcher = FieldMatcher::new("!012A?");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(matcher.is_match(&field!("013A", '0', "abc"), &options));

    // exists
    let matcher = FieldMatcher::new("!!012A?");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&field!("012A", '0', "abc"), &options));
    assert!(!matcher.is_match(&field!("013A", '0', "abc"), &options));
}

#[test]
fn field_matcher_and() {
    let options = MatcherOptions::default();
    let matcher = FieldMatcher::new(
        "012A? && #014A == 0 && 013A{#a == 1 && a == '123'}",
    );

    assert!(matcher.is_match(
        vec![&field!("012A", '0', "abc"), &field!("013A", 'a', "123")],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &field!("012A", '0', "abc"),
            &field!("013A", 'a', "123"),
            &field!("014A", '0', "hij"),
        ],
        &options
    ));
}

#[test]
fn field_matcher_composite_or() {
    let matcher =
        FieldMatcher::new("012A? || 013A{#a == 1 && a == '1'}");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &field!("012A", '0', "abc"),
            &FieldRef::new("013A", None, vec![('a', "1"), ('a', "2")]),
        ],
        &options
    ));

    assert!(matcher.is_match(
        vec![&field!("013A", 'a', "1"), &field!("014A", '0', "abc")],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &FieldRef::new("013A", None, vec![('a', "1"), ('a', "2")]),
            &FieldRef::new("014A", None, vec![('0', "abc")]),
        ],
        &options
    ));

    let matcher =
        FieldMatcher::new("!014A.x? || 013A{#a == 2 && a == '1'}");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &FieldRef::new("012A", None, vec![('0', "abc")]),
            &FieldRef::new("013A", None, vec![('a', "1"), ('a', "2")]),
        ],
        &options
    ));
}
