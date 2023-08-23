use pica_matcher::{FieldMatcher, MatcherOptions};
use pica_record::FieldMut;

#[test]
fn field_matcher_exists() -> anyhow::Result<()> {
    let matcher = FieldMatcher::new("003@?")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("003@", None, vec![('0', "123456789X")]),
        &options
    ));

    assert!(!matcher.is_match(
        &FieldMut::new("002@", None, vec![('0', "Olfo")]),
        &options
    ));

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("002@", None, vec![('0', "Olfo")]),
            &FieldMut::new("003@", None, vec![('0', "123456789X")]),
        ],
        &options
    ));

    let matcher = FieldMatcher::new("00[23]@?")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("003@", None, vec![('0', "123456789X")]),
        &options
    ));

    assert!(matcher.is_match(
        &FieldMut::new("002@", None, vec![('0', "Olfo")]),
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_subfields() -> anyhow::Result<()> {
    // simple
    let matcher = FieldMatcher::new("003@.0 == '123456789X'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("003@", None, vec![('0', "123456789X")]),
        &options
    ));

    assert!(!matcher.is_match(
        &FieldMut::new("002@", None, vec![('0', "Olfo")]),
        &options
    ));

    // complex
    let matcher = FieldMatcher::new("003@{0? && 0 == '123456789X'}")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("003@", None, vec![('0', "123456789X")]),
        &options
    ));

    assert!(!matcher.is_match(
        &FieldMut::new("003@", None, vec![('0', "34567")]),
        &options
    ));

    assert!(!matcher.is_match(
        &FieldMut::new("002@", None, vec![('0', "Olfo")]),
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_cardinality_eq() -> anyhow::Result<()> {
    let matcher = FieldMatcher::new("#012A == 1")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('0', "def")]),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_cardinality_ne() -> anyhow::Result<()> {
    let matcher = FieldMatcher::new("#012A{0 =^ 'ab'} != 1")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('0', "abd")]),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_cardinality_ge() -> anyhow::Result<()> {
    let matcher = FieldMatcher::new("#012A >= 2")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('0', "def")]),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_cardinality_gt() -> anyhow::Result<()> {
    let matcher = FieldMatcher::new("#012A{ 0 =^ 'ab' } > 1")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('0', "def")]),
        ],
        &options
    ));

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('X', "def")]),
            &FieldMut::new("012A", None, vec![('0', "abd")]),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_cardinality_le() -> anyhow::Result<()> {
    let matcher = FieldMatcher::new("#012A <= 2")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('0', "def")]),
        ],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('0', "def")]),
            &FieldMut::new("012A", None, vec![('0', "hij")]),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_cardinality_lt() -> anyhow::Result<()> {
    let matcher = FieldMatcher::new("#012A{ 0 =^ 'ab' } < 2")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('0', "def")]),
        ],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("012A", None, vec![('X', "def")]),
            &FieldMut::new("012A", None, vec![('0', "abd")]),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_group() -> anyhow::Result<()> {
    // singleton
    let matcher = FieldMatcher::new("(012A?)")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(!matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "abc")]),
        &options
    ));

    // not
    let matcher = FieldMatcher::new("(!012A?)")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "abc")]),
        &options
    ));

    // cardinality
    let matcher = FieldMatcher::new("(#012A <= 1)")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "abc")]),
        &options
    ));

    // group
    let matcher = FieldMatcher::new("((012A?))")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(!matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "abc")]),
        &options
    ));

    // and
    let matcher = FieldMatcher::new("(012A? && 012A.0 == 'abc')")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(!matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "def")]),
        &options
    ));

    // or
    let matcher = FieldMatcher::new("(012A? || 013A.0 == 'abc')")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(!matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "def")]),
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_not() -> anyhow::Result<()> {
    // Group
    let matcher = FieldMatcher::new("!(012A?)")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "abc")]),
        &options
    ));

    // exists
    let matcher = FieldMatcher::new("!012A?")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "abc")]),
        &options
    ));

    // exists
    let matcher = FieldMatcher::new("!!012A?")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        &FieldMut::new("012A", None, vec![('0', "abc")]),
        &options
    ));
    assert!(!matcher.is_match(
        &FieldMut::new("013A", None, vec![('0', "abc")]),
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_composite_and() -> anyhow::Result<()> {
    let matcher = FieldMatcher::new(
        "012A? && #014A == 0 && 013A{#a == 1 && a == '123'}",
    )?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("013A", None, vec![('a', "123")]),
        ],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("013A", None, vec![('a', "123")]),
            &FieldMut::new("014A", None, vec![('0', "hij")]),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn field_matcher_composite_or() -> anyhow::Result<()> {
    let matcher =
        FieldMatcher::new("012A? || 013A{#a == 1 && a == '1'}")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("013A", None, vec![('a', "1"), ('a', "2")]),
        ],
        &options
    ));

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("013A", None, vec![('a', "1")]),
            &FieldMut::new("014A", None, vec![('0', "abc")]),
        ],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &FieldMut::new("013A", None, vec![('a', "1"), ('a', "2")]),
            &FieldMut::new("014A", None, vec![('0', "abc")]),
        ],
        &options
    ));

    let matcher =
        FieldMatcher::new("!014A.x? || 013A{#a == 2 && a == '1'}")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &FieldMut::new("012A", None, vec![('0', "abc")]),
            &FieldMut::new("013A", None, vec![('a', "1"), ('a', "2")]),
        ],
        &options
    ));

    Ok(())
}
