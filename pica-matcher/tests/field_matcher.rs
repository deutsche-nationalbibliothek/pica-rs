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
fn field_matcher_simple() -> anyhow::Result<()> {
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

    Ok(())
}

#[test]
fn field_matcher_complex() -> anyhow::Result<()> {
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
