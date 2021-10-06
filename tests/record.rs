use pica::{Field, Occurrence, Subfield, Tag};

#[test]
fn subfield_new() {
    assert!(Subfield::new('0', "12283643X").is_ok());

    assert_eq!(
        format!("{}", Subfield::new('!', "12283643X").unwrap_err()),
        "Invalid subfield code '!'"
    );

    assert_eq!(
        format!("{}", Subfield::new('0', "12283\x1e643X").unwrap_err()),
        "Invalid subfield value."
    );

    assert_eq!(
        format!("{}", Subfield::new('0', "12283\x1f643X").unwrap_err()),
        "Invalid subfield value."
    );
}

#[test]
fn subfield_validate() {
    assert_eq!(
        format!(
            "{}",
            Subfield::new('0', vec![0, 159])
                .unwrap()
                .validate()
                .unwrap_err()
        ),
        "invalid utf-8 sequence of 1 bytes from index 1"
    );
}

#[test]
fn occurrence_new() {
    assert!(Occurrence::new("01").is_ok());
    assert!(Occurrence::new("001").is_ok());

    assert_eq!(
        format!("{}", Occurrence::new("0001").unwrap_err()),
        "length < 2 || length > 3"
    );

    assert_eq!(
        format!("{}", Occurrence::new("0a").unwrap_err()),
        "Invalid occurrence '0a'"
    );
}

#[test]
fn field_new() {
    assert!(Field::new(Tag::new("003@").unwrap(), None, vec![]).is_ok());
}
