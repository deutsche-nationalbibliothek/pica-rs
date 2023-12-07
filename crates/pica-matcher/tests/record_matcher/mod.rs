use std::path::Path;
use std::str::FromStr;
use std::sync::OnceLock;
use std::{env, fs};

use bstr::B;
use pica_matcher::{MatcherOptions, ParseMatcherError, RecordMatcher};
use pica_record::RecordRef;

use crate::TestResult;

fn ada_lovelace() -> &'static [u8] {
    static DATA: OnceLock<Vec<u8>> = OnceLock::new();
    DATA.get_or_init(|| {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&manifest_dir)
            .join("../pica-toolkit/tests/data/119232022.dat");
        eprintln!("{:?}", path);
        fs::read_to_string(&path).unwrap().as_bytes().to_vec()
    })
}

#[test]
fn record_matcher_new() -> TestResult {
    let matcher = RecordMatcher::new("003@.0?");

    assert!(matcher.is_match(
        &RecordRef::from_bytes(ada_lovelace())?,
        &MatcherOptions::default()
    ));

    Ok(())
}

#[test]
#[should_panic]
fn record_matcher_new_panic() {
    let _ = RecordMatcher::new("003@.!?");
}

#[test]
fn record_matcher_try_from() -> TestResult {
    let matcher = RecordMatcher::try_from(B("003@.0?"))?;

    assert!(matcher.is_match(
        &RecordRef::from_bytes(ada_lovelace())?,
        &MatcherOptions::default()
    ));

    assert!(matches!(
        RecordMatcher::try_from(B("003@.!?")).unwrap_err(),
        ParseMatcherError::InvalidRecordMatcher(_)
    ));

    Ok(())
}

#[test]
fn record_matcher_from_str() -> TestResult {
    let matcher = RecordMatcher::from_str("003@.0?")?;

    assert!(matcher.is_match(
        &RecordRef::from_bytes(ada_lovelace())?,
        &MatcherOptions::default()
    ));

    assert!(matches!(
        RecordMatcher::from_str("003@.!?").unwrap_err(),
        ParseMatcherError::InvalidRecordMatcher(_)
    ));

    Ok(())
}

#[test]
fn record_matcher_exists() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;

    let matcher = RecordMatcher::new("004B?");
    assert!(matcher.is_match(&record, &Default::default()));

    let matcher = RecordMatcher::new("028A.a?");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_cardinality() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher = RecordMatcher::new(
        "#028[A@]{d =^ 'Ada' && a == 'Lovelace'} == 5",
    );

    assert!(matcher.is_match(&record, &Default::default()));
    Ok(())
}

#[test]
fn record_matcher_in() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher = RecordMatcher::new("002@.0 in ['Tpz', 'Tp1']");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_regex() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher = RecordMatcher::new("047A/03.[er] =~ '^DE-\\\\d+6'");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_eq() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher = RecordMatcher::new("003@.0 == '119232022'");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_not_equal() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher = RecordMatcher::new("002@.0 != 'Ts1'");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_starts_with() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher =
        RecordMatcher::new("003U.a =^ 'http://d-nb.info/gnd/'");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_ends_with() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher = RecordMatcher::new("042B.a =$ '-GB'");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_group() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher =
        RecordMatcher::new("(002@.0 == 'Tp1' && 004B.a == 'pik')");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_not() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher =
        RecordMatcher::new("!(002@.0 == 'Ts1' || 002@.0 =^ 'Tu')");
    assert!(matcher.is_match(&record, &Default::default()));

    let matcher = RecordMatcher::new("!012A.0?");
    assert!(matcher.is_match(&record, &Default::default()));
    Ok(())
}

#[test]
fn record_matcher_and() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher =
        RecordMatcher::new("002@.0 == 'Tp1' && 004B.a == 'pik'");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}

#[test]
fn record_matcher_or() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let matcher =
        RecordMatcher::new("002@.0 == 'Ts1' || 004B.a == 'pik'");
    assert!(matcher.is_match(&record, &Default::default()));

    Ok(())
}
