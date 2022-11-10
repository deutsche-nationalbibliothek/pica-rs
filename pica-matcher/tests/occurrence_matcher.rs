use pica_matcher::OccurrenceMatcher;
use pica_record::OccurrenceMut;

#[test]
fn test_occurrence_matcher_eq() -> anyhow::Result<()> {
    let matcher = OccurrenceMatcher::new("/02")?;

    assert!(!matcher.is_match(&OccurrenceMut::new("01")));
    assert!(matcher.is_match(&OccurrenceMut::new("02")));
    assert!(!matcher.is_match(&OccurrenceMut::new("03")));

    Ok(())
}

#[test]
fn test_occurrence_matcher_range() -> anyhow::Result<()> {
    let matcher = OccurrenceMatcher::new("/01-03")?;

    assert!(matcher.is_match(&OccurrenceMut::new("01")));
    assert!(matcher.is_match(&OccurrenceMut::new("02")));
    assert!(matcher.is_match(&OccurrenceMut::new("03")));

    assert!(!matcher.is_match(&OccurrenceMut::new("00")));
    assert!(!matcher.is_match(&OccurrenceMut::new("001")));
    assert!(!matcher.is_match(&OccurrenceMut::new("04")));

    Ok(())
}

#[test]
fn test_occurrence_matcher_any() -> anyhow::Result<()> {
    let matcher = OccurrenceMatcher::new("/*")?;
    assert!(matcher.is_match(&OccurrenceMut::new("01")));
    assert!(matcher.is_match(&OccurrenceMut::new("00")));
    assert!(matcher.is_match(&OccurrenceMut::new("001")));
    Ok(())
}
