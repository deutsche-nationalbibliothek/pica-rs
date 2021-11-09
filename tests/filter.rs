use pica::{Occurrence, OccurrenceMatcher};

#[test]
fn test_occurrence_matcher() {
    // OccurrenceMatcher::Any
    assert_eq!(Some(Occurrence::new("01").unwrap()), OccurrenceMatcher::Any);
    assert_eq!(Option::<&Occurrence>::None, OccurrenceMatcher::Any);

    // OccurrenceMatcher::None
    assert_eq!(Option::<&Occurrence>::None, OccurrenceMatcher::None);
    assert_ne!(
        Some(Occurrence::new("01").unwrap()),
        OccurrenceMatcher::None
    );

    // OccurrenceMatcher::Occurrence
    let matcher = OccurrenceMatcher::Some(Occurrence::new("01").unwrap());
    assert_eq!(Some(Occurrence::new("01").unwrap()), matcher);
    assert_ne!(Some(Occurrence::new("02").unwrap()), matcher);
    assert_ne!(Option::<&Occurrence>::None, matcher);

    // OccurrenceMatcher::Range
    let matcher = OccurrenceMatcher::range("02", "04").unwrap();
    assert_ne!(Some(Occurrence::new("01").unwrap()), matcher);
    assert_eq!(Some(Occurrence::new("02").unwrap()), matcher);
    assert_eq!(Some(Occurrence::new("03").unwrap()), matcher);
    assert_eq!(Some(Occurrence::new("04").unwrap()), matcher);
    assert_ne!(Some(Occurrence::new("05").unwrap()), matcher);
    assert_ne!(Option::<&Occurrence>::None, matcher);
}
