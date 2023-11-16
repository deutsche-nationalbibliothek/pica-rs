use pica_matcher::TagMatcher;
use pica_record::Tag;

#[test]
fn tag_matcher_simple() -> anyhow::Result<()> {
    let matcher = TagMatcher::new("003@");

    assert!(matcher.is_match(&Tag::new("003@")));
    assert!(!matcher.is_match(&Tag::new("002@")));

    Ok(())
}

#[test]
fn tag_matcher_pattern() -> anyhow::Result<()> {
    let matcher = TagMatcher::new("00[23]@");

    assert!(matcher.is_match(&Tag::new("002@")));
    assert!(matcher.is_match(&Tag::new("003@")));
    assert!(!matcher.is_match(&Tag::new("004@")));

    let matcher = TagMatcher::new("01[2-4]A");
    assert!(!matcher.is_match(&Tag::new("011A")));
    assert!(matcher.is_match(&Tag::new("012A")));
    assert!(matcher.is_match(&Tag::new("013A")));
    assert!(matcher.is_match(&Tag::new("014A")));
    assert!(!matcher.is_match(&Tag::new("015A")));

    let matcher = TagMatcher::new("0..A");
    assert!(matcher.is_match(&Tag::new("011A")));
    assert!(matcher.is_match(&Tag::new("022A")));
    assert!(!matcher.is_match(&Tag::new("123A")));

    Ok(())
}
