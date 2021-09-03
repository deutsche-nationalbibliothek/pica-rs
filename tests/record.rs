use pica::Subfield;

#[test]
fn subfield_new() {
    assert!(Subfield::new('0', "12283643X").is_ok());
    assert!(Subfield::new('!', "12283643X").is_err());
    assert!(Subfield::new('a', "123\x1f34").is_err());
    assert!(Subfield::new('a', "123\x1e34").is_err());
}
