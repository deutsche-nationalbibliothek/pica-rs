extern crate pica;

use pica::Filter;
use pica::ParsePicaError;
use pica::Record;

const RECORD_STR: &'static str = concat!(
    "003@ \u{1f}0123456789X\u{1e}",
    "002@ \u{1f}0Tp3\u{1e}",
    "010@ \u{1f}ager\u{1f}aeng\u{1e}"
);

#[test]
fn parse_filter() {
    let result = "003@.0?".parse::<Filter>();
    assert!(result.is_ok());

    let result = "003@.0~".parse::<Filter>();
    assert_eq!(result.err(), Some(ParsePicaError::InvalidFilter));
}

#[test]
fn comp_eq_filter() {
    let record = RECORD_STR.parse::<Record>().unwrap();

    let filter = "003@.0 == '123456789X'".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));

    let filter = "003@.0 == '23456789X1'".parse::<Filter>().unwrap();
    assert!(!record.matches(&filter));

    let filter = "010@.a == 'ger'".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));
}

#[test]
fn comp_ne_filter() {
    let record = RECORD_STR.parse::<Record>().unwrap();

    let filter = "003@.0 != '23456789X1'".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));

    let filter = "003@.0 != '123456789X'".parse::<Filter>().unwrap();
    assert!(!record.matches(&filter));

    let filter = "010@.a != 'ger'".parse::<Filter>().unwrap();
    assert!(!record.matches(&filter));
}

#[test]
fn regex_filter() {
    let record = RECORD_STR.parse::<Record>().unwrap();
    let filter = "002@.0 =~ '^Tp[123]$'".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));
}

#[test]
fn filter_exists() {
    let record = RECORD_STR.parse::<Record>().unwrap();

    let filter = "002@.0?".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));

    let filter = "006U.0?".parse::<Filter>().unwrap();
    assert!(!record.matches(&filter));
}

#[test]
fn filter_boolean_and() {
    let record = RECORD_STR.parse::<Record>().unwrap();

    let filter = "002@.0 =~ '^T[pf]3$' && 010@.a == 'eng'"
        .parse::<Filter>()
        .unwrap();
    assert!(record.matches(&filter));

    let filter = "002@.0 =~ '^T[pf]3$' && 010@.a == 'rus'"
        .parse::<Filter>()
        .unwrap();
    assert!(!record.matches(&filter));
}

#[test]
fn filter_boolean_or() {
    let record = RECORD_STR.parse::<Record>().unwrap();

    let filter = "002@.0 =~ '^T[pf]4$' || 010@.a == 'eng'"
        .parse::<Filter>()
        .unwrap();
    assert!(record.matches(&filter));

    let filter = "002@.0 =~ '^T[pf]3$' || 010@.a == 'rus'"
        .parse::<Filter>()
        .unwrap();
    assert!(record.matches(&filter));

    let filter = "002@.0 == 'Tp4' || 010@.a == 'rus'"
        .parse::<Filter>()
        .unwrap();
    assert!(!record.matches(&filter));
}

#[test]
fn filter_grouped() {
    let record = RECORD_STR.parse::<Record>().unwrap();

    let filter = "(002@.0 =~ '^T[pf]3$')".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));

    let filter = "(003@.0? && 002@.0 == 'Tp3')".parse::<Filter>().unwrap();
    assert!(record.matches(&filter));

    let filter = "(002@.0 == 'Tp2' || 002@.0 == 'Tp3') && 010@.a == 'eng'"
        .parse::<Filter>()
        .unwrap();
    assert!(record.matches(&filter));

    let filter = concat!(
        "(002@.0 == 'Tp2' || 002@.0 == 'Tp3') && ",
        "(010@.a == 'eng' || 010@.a == 'ger')"
    )
    .parse::<Filter>()
    .unwrap();

    assert!(record.matches(&filter));
}
