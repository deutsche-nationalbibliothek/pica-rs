use pica_matcher::OccurrenceMatcher;
use pica_record::OccurrenceRef;

#[test]
fn test_occurrence_matcher_eq() {
    let matcher = OccurrenceMatcher::new("/02");

    assert!(!matcher.is_match(&OccurrenceRef::new("01")));
    assert!(matcher.is_match(&OccurrenceRef::new("02")));
    assert!(!matcher.is_match(&OccurrenceRef::new("03")));
}

#[test]
fn test_occurrence_matcher_range() {
    let matcher = OccurrenceMatcher::new("/01-03");

    assert!(matcher.is_match(&OccurrenceRef::new("01")));
    assert!(matcher.is_match(&OccurrenceRef::new("02")));
    assert!(matcher.is_match(&OccurrenceRef::new("03")));

    assert!(!matcher.is_match(&OccurrenceRef::new("00")));
    assert!(!matcher.is_match(&OccurrenceRef::new("001")));
    assert!(!matcher.is_match(&OccurrenceRef::new("04")));
}

#[test]
fn test_occurrence_matcher_any() {
    let matcher = OccurrenceMatcher::new("/*");
    assert!(matcher.is_match(&OccurrenceRef::new("01")));
    assert!(matcher.is_match(&OccurrenceRef::new("00")));
    assert!(matcher.is_match(&OccurrenceRef::new("001")));
}
