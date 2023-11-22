use pica_matcher::TagMatcher;
use pica_record::TagRef;

const TAGS: [&'static str; 24] = [
    "001A", "001B", "001D", "001U", "001X", "002@", "003@", "003U",
    "004B", "007K", "007N", "008A", "008B", "010E", "028@", "028A",
    "028R", "032T", "041R", "042A", "042B", "047A", "047C", "050C",
];

#[test]
fn tag_matcher_new() {
    let matcher = TagMatcher::new("003@");
    assert!(matcher.is_match(&TagRef::new("003@")));
    assert!(!matcher.is_match(&TagRef::new("002@")));

    let matcher = TagMatcher::new("01[2-4]A");
    assert!(!matcher.is_match(&TagRef::new("011A")));
    assert!(matcher.is_match(&TagRef::new("012A")));
    assert!(matcher.is_match(&TagRef::new("013A")));
    assert!(matcher.is_match(&TagRef::new("014A")));
    assert!(!matcher.is_match(&TagRef::new("015A")));
}

#[test]
#[should_panic]
fn tag_matcher_new_panic() {
    let _matcher = TagMatcher::new("[0-5]03@");
}

#[test]
fn tag_matcher_is_match() {
    for tag in TAGS {
        let matcher = TagMatcher::new(tag);
        assert!(matcher.is_match(&TagRef::new(tag)));

        let matcher = TagMatcher::new("....");
        assert!(matcher.is_match(&TagRef::new(tag)));

        let matcher =
            TagMatcher::new("[0-2][0-5][01-78][ABDUX@KNECRT]");
        assert!(matcher.is_match(&TagRef::new(tag)));
    }
}

#[test]
fn tag_matcher_partial_eq() {
    for tag in TAGS {
        assert_eq!(TagRef::new(tag), TagMatcher::new(tag));
        assert_eq!(TagMatcher::new(tag), TagRef::new(tag));
        assert_eq!(TagRef::new(tag), TagMatcher::new("...."));
        assert_eq!(TagMatcher::new("...."), TagRef::new(tag));
        assert_eq!(
            TagRef::new(tag),
            TagMatcher::new("[0-2][0-5][01-78][ABDUX@KNECRT]")
        );
    }
}

#[test]
fn tag_matcher_from_str() {
    for tag in TAGS {
        let matcher = tag.parse::<TagMatcher>().unwrap();
        assert!(matcher.is_match(&TagRef::new(tag)));
    }
}
