use pica_record::{Level, ParsePicaError, Tag, TagRef};

#[test]
fn tag_ref_new() {
    assert_eq!(TagRef::new(b"044H"), "044H");
    assert_eq!(TagRef::new("044H"), "044H");
}

#[test]
#[should_panic]
fn tag_ref_new_panic() {
    let _tag = TagRef::new("303@");
}

#[test]
fn tag_ref_from_bytes() {
    let tag = TagRef::from_bytes(b"003@").unwrap();
    assert_eq!(tag, "003@");

    let err = TagRef::from_bytes(b"303@").unwrap_err();
    assert_eq!(err, ParsePicaError::InvalidTag);
}

#[test]
fn tag_ref_try_from() {
    let tag = TagRef::try_from("003@".as_bytes()).unwrap();
    assert_eq!(tag, "003@");

    let err = TagRef::try_from("303@".as_bytes()).unwrap_err();
    assert_eq!(err, ParsePicaError::InvalidTag);
}

#[test]
fn tag_ref_level() {
    assert_eq!(TagRef::new("003@").level(), Level::Main);
    assert_eq!(TagRef::new("101@").level(), Level::Local);
    assert_eq!(TagRef::new("203@").level(), Level::Copy);
}

#[test]
fn tag_ref_index() {
    let tag = TagRef::new("012A");
    assert_eq!(tag[0], b'0');
    assert_eq!(tag[1], b'1');
    assert_eq!(tag[2], b'2');
    assert_eq!(tag[3], b'A');
}

#[test]
#[should_panic]
fn tag_ref_index_panic() {
    let tag = TagRef::new("012A");
    assert_eq!(tag[4], b'A');
}

#[test]
fn tag_ref_to_string() {
    let tag = TagRef::new("041A");
    assert_eq!(tag.to_string(), "041A".to_string());
}

#[test]
fn tag_new() {
    assert_eq!(Tag::new(b"044H"), "044H");
    assert_eq!(Tag::new("044H"), "044H");
}

#[test]
fn tag_from_tag_ref() {
    let tag_ref = TagRef::new("041A");
    let tag = Tag::new("041A");

    assert_eq!(Tag::from(tag_ref), tag);
}

#[test]
fn tag_partial_eq() {
    let tag_ref = TagRef::new("041A");
    let tag = Tag::new("041A");

    assert_eq!(tag_ref, b"041A");
    assert_eq!(tag, b"041A");
    assert_eq!(tag, tag_ref);
    assert_eq!(tag_ref, tag);

    let tag_ref = TagRef::new("041A");
    let tag = Tag::new("044H");

    assert_ne!(tag_ref, b"044H");
    assert_ne!(tag, b"041A");
    assert_ne!(tag, tag_ref);
    assert_ne!(tag_ref, tag);
}
