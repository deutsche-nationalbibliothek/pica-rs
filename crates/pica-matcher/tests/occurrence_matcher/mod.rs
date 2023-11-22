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

#[test]
fn is_match() {
    let matcher = OccurrenceMatcher::new("/01");
    assert!(!matcher.is_match(&OccurrenceRef::new("00")));
    assert!(matcher.is_match(&OccurrenceRef::new("01")));

    let matcher = OccurrenceMatcher::new("/01-03");
    assert!(!matcher.is_match(&OccurrenceRef::new("00")));
    assert!(matcher.is_match(&OccurrenceRef::new("01")));
    assert!(matcher.is_match(&OccurrenceRef::new("02")));
    assert!(matcher.is_match(&OccurrenceRef::new("03")));
    assert!(!matcher.is_match(&OccurrenceRef::new("04")));

    let matcher = OccurrenceMatcher::new("/*");
    assert!(matcher.is_match(&OccurrenceRef::new("00")));
    assert!(matcher.is_match(&OccurrenceRef::new("01")));

    let matcher = OccurrenceMatcher::new("/00");
    assert!(matcher.is_match(&OccurrenceRef::new("00")));
    assert!(!matcher.is_match(&OccurrenceRef::new("01")));
}

#[test]
fn test_partial_eq() {
    let matcher = OccurrenceMatcher::new("/01");
    assert_ne!(matcher, OccurrenceRef::new("00"));
    assert_eq!(matcher, OccurrenceRef::new("01"));
    assert_ne!(matcher, Option::<OccurrenceRef>::None.as_ref());

    let matcher = OccurrenceMatcher::new("/01-03");
    assert_ne!(matcher, OccurrenceRef::new("00"));
    assert_eq!(matcher, OccurrenceRef::new("01"));
    assert_eq!(matcher, OccurrenceRef::new("02"));
    assert_eq!(matcher, OccurrenceRef::new("03"));
    assert_ne!(matcher, OccurrenceRef::new("04"));
    assert_ne!(matcher, Option::<OccurrenceRef>::None.as_ref());

    let matcher = OccurrenceMatcher::new("/*");
    assert_eq!(matcher, OccurrenceRef::new("000"));
    assert_eq!(matcher, OccurrenceRef::new("00"));
    assert_eq!(matcher, OccurrenceRef::new("001"));
    assert_eq!(matcher, OccurrenceRef::new("01"));
    assert_eq!(matcher, Option::<OccurrenceRef>::None.as_ref());

    let matcher = OccurrenceMatcher::new("/00");
    assert_eq!(matcher, OccurrenceRef::new("00"));
    assert_ne!(matcher, OccurrenceRef::new("01"));
    assert_eq!(matcher, Option::<OccurrenceRef>::None.as_ref());
}
