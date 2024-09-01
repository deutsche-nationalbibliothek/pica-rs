use pica_record::Level;

#[test]
fn level_from_str() {
    assert_eq!("main".parse::<Level>().unwrap(), Level::Main);
    assert_eq!("local".parse::<Level>().unwrap(), Level::Local);
    assert_eq!("copy".parse::<Level>().unwrap(), Level::Copy);

    let err = "master".parse::<Level>().unwrap_err();
    assert_eq!(err.to_string(), "invalid level 'master'");
}

#[test]
fn level_default() {
    assert_eq!(Level::default(), Level::Main);
}
