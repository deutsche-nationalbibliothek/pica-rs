use pica_matcher::subfield_matcher::{
    ExistsMatcher, Matcher, RegexMatcher, RelationMatcher,
};
use pica_matcher::MatcherOptions;
use pica_record::SubfieldRef;

#[test]
fn exists_matcher() -> anyhow::Result<()> {
    let matcher = ExistsMatcher::new("1?")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f1hij")?,
        ],
        &options
    ));

    for matcher_str in ["[a12]?", "a12?"] {
        let matcher = ExistsMatcher::new(matcher_str)?;
        let options = MatcherOptions::default();

        assert!(matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f1abc")?,
            &options
        ));
        assert!(!matcher.is_match(
            &SubfieldRef::from_bytes(b"\x1f9abc")?,
            &options
        ));
        assert!(matcher.is_match(
            [
                &SubfieldRef::from_bytes(b"\x1f3def")?,
                &SubfieldRef::from_bytes(b"\x1f9hij")?,
                &SubfieldRef::from_bytes(b"\x1f2bsg")?,
            ],
            &options
        ));
    }

    Ok(())
}

#[test]
fn relational_matcher_eq() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RelationMatcher::new("0 == 'abc'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &options));
    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    // case insensitive
    let matcher = RelationMatcher::new("0 == 'abc'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 == 'abc'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn relational_matcher_ne() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RelationMatcher::new("0 != 'abc'")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &options));
    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    // case insensitive
    let matcher = RelationMatcher::new("0 != 'abc'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 != 'abc'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f0bsg")?,
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn relational_matcher_starts_with() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RelationMatcher::new("0 =^ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0def")?, &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 =^ 'ab'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0aBc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0def")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 =^ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0baab")?,
            &SubfieldRef::from_bytes(b"\x1f0abba")?,
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f1abc")?,
        ],
        &options
    ));
    Ok(())
}

#[test]
fn relational_matcher_ends_with() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RelationMatcher::new("0 =$ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abab")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abba")?, &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 =$ 'ab'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abab")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abAB")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 =$ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0baba")?,
            &SubfieldRef::from_bytes(b"\x1f0abab")?,
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f1aab")?,
        ],
        &options
    ));
    Ok(())
}

#[test]
fn relational_matcher_similar() -> anyhow::Result<()> {
    // default threshold
    let matcher = RelationMatcher::new("a =* 'Heike'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faHeike")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faHeiko")?, &options));

    // threshold set
    let matcher = RelationMatcher::new("a =* 'Heike'")?;
    let options = MatcherOptions::new().strsim_threshold(0.7);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faHeike")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faHeiko")?, &options));

    // default threshold
    let matcher = RelationMatcher::new("a =* 'Heike'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faheike")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("a =* 'Heike'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1faHeiko")?,
            &SubfieldRef::from_bytes(b"\x1faHeike")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn regex_matcher() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RegexMatcher::new("0 =~ '^ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abba")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABBA")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabba")?, &options));

    // case insensitive
    let matcher = RegexMatcher::new("0 =~ '^ab'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abba")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABBA")?, &options));

    // invert match
    let matcher = RegexMatcher::new("0 !~ '^ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0baba")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abba")?, &options));

    // multiple subfields
    let matcher = RegexMatcher::new("0 =~ '^ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0foobar")?,
            &SubfieldRef::from_bytes(b"\x1f0abba")?
        ],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0foo")?,
            &SubfieldRef::from_bytes(b"\x1f0bar")?
        ],
        &options
    ));

    Ok(())
}
