use pica_matcher::TagMatcher;
use pica_record::TagRef;

#[test]
fn tag_matcher_simple() -> anyhow::Result<()> {
    let matcher = TagMatcher::new("003@");

    assert!(matcher.is_match(&TagRef::new("003@")));
    assert!(!matcher.is_match(&TagRef::new("002@")));

    Ok(())
}

#[test]
fn tag_matcher_pattern() -> anyhow::Result<()> {
    let matcher = TagMatcher::new("00[23]@");

    assert!(matcher.is_match(&TagRef::new("002@")));
    assert!(matcher.is_match(&TagRef::new("003@")));
    assert!(!matcher.is_match(&TagRef::new("004@")));

    let matcher = TagMatcher::new("01[2-4]A");
    assert!(!matcher.is_match(&TagRef::new("011A")));
    assert!(matcher.is_match(&TagRef::new("012A")));
    assert!(matcher.is_match(&TagRef::new("013A")));
    assert!(matcher.is_match(&TagRef::new("014A")));
    assert!(!matcher.is_match(&TagRef::new("015A")));

    let matcher = TagMatcher::new("0..A");
    assert!(matcher.is_match(&TagRef::new("011A")));
    assert!(matcher.is_match(&TagRef::new("022A")));
    assert!(!matcher.is_match(&TagRef::new("123A")));

    Ok(())
}
