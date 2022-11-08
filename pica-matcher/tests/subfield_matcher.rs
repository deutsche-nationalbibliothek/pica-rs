use pica_matcher::subfield_matcher::{ExistsMatcher, Matcher};
use pica_matcher::MatcherOptions;
use pica_record::SubfieldRef;

#[test]
fn test_exists_matcher() -> anyhow::Result<()> {
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
