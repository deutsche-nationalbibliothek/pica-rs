use pica::matcher::OccurrenceMatcher;
use pica::Path;

#[test]
fn path_new() {
    assert!(
        Path::new("003@", OccurrenceMatcher::None, vec!['0']).is_ok()
    );
    assert!(
        Path::new("303@", OccurrenceMatcher::None, vec!['0']).is_err()
    );
    assert!(
        Path::new("003@", OccurrenceMatcher::None, vec!['!']).is_err()
    );
}

#[test]
fn path_from_bytes() {
    assert!(Path::from_bytes("012A/*.0").is_ok());

    assert_eq!(
        format!("{}", Path::from_bytes("312A.0").unwrap_err()),
        "Invalid path expression"
    );

    assert_eq!(
        format!("{}", Path::from_bytes("312A/!.0").unwrap_err()),
        "Invalid path expression"
    );

    assert_eq!(
        format!("{}", Path::from_bytes("012A/*.!").unwrap_err()),
        "Invalid path expression"
    );
}
