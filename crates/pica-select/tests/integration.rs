use std::str::FromStr;
use std::sync::OnceLock;

use bstr::B;
use pica_format::Format;
use pica_path::Path;
use pica_record_v1::RecordRef;
use pica_select::{ParseQueryError, Query, QueryExt, QueryOptions};

type TestResult = anyhow::Result<()>;

fn ada_lovelace() -> &'static [u8] {
    use std::path::Path;
    use std::{env, fs};

    static DATA: OnceLock<Vec<u8>> = OnceLock::new();
    DATA.get_or_init(|| {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&manifest_dir)
            .join("../pica-toolkit/tests/data/119232022.dat");
        fs::read_to_string(&path).unwrap().as_bytes().to_vec()
    })
}

#[test]
fn query_new() -> TestResult {
    let query = Query::new("003@.0");
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let options = QueryOptions::default();

    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022"]]
    );

    Ok(())
}

#[test]
#[should_panic]
fn query_new_panic() {
    let _ = Query::new("003@.!");
}

#[test]
fn query_try_from() -> TestResult {
    let query = Query::try_from(B("003@.0"))?;
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let options = QueryOptions::default();

    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022"]]
    );

    assert!(matches!(
        Query::try_from(B("003@.!")).unwrap_err(),
        ParseQueryError(_)
    ));

    Ok(())
}

#[test]
fn query_from_str() -> TestResult {
    let query = Query::from_str("003@.0")?;
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let options = QueryOptions::default();

    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022"]]
    );

    assert!(matches!(
        Query::from_str("003@.!").unwrap_err(),
        ParseQueryError(_)
    ));

    Ok(())
}

#[test]
fn query_from_path() -> TestResult {
    let query = Query::from(Path::new("003@.0"));
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let options = QueryOptions::default();

    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022"]]
    );

    Ok(())
}

#[test]
fn query_from_format() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let query = Query::from(Format::new("028A{ a <$> (', ' d) }"));
    let options = QueryOptions::default();

    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["Lovelace, Ada King"]]
    );

    Ok(())
}

#[test]
fn record_query_default() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let query = Query::new("065R{ (9,7) | 4 == 'ortg'}");
    let options = QueryOptions::default();

    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["040743357", "Tgz"]]
    );

    Ok(())
}

#[test]
fn record_query_case_ignore() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let query = Query::new("028R{ d, a | a == 'KING' }");

    let options = QueryOptions::default().case_ignore(true);
    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["william", "king"]]
    );

    let options = QueryOptions::default().case_ignore(false);
    assert_eq!(record.query(&query, &options).as_ref(), [["", ""]]);

    Ok(())
}

#[test]
fn record_query_squash() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let query = Query::new("008A.a");

    let options = QueryOptions::default().squash(true);
    assert_eq!(record.query(&query, &options).as_ref(), [["s|z|f"]]);

    let options = QueryOptions::default().squash(false);
    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["s"], ["z"], ["f"]]
    );

    Ok(())
}

#[test]
fn record_query_merge() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let query = Query::new("003@.0, 008A.a");

    let options = QueryOptions::default().merge(true);
    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022", "s|z|f"]]
    );

    let options = QueryOptions::default().merge(false);
    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022", "s"], ["119232022", "z"], ["119232022", "f"]]
    );

    Ok(())
}

#[test]
fn record_query_separator() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let query = Query::new("003@.0, 008A.a");

    let options = QueryOptions::default().squash(true).separator("+");
    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022", "s+z+f"]]
    );

    let options = QueryOptions::default().merge(true).separator("+");
    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022", "s+z+f"]]
    );

    Ok(())
}

#[test]
fn record_query_const() -> TestResult {
    let record = RecordRef::from_bytes(ada_lovelace())?;
    let query = Query::new("003@.0, 'abc', 003@.0");
    let options = QueryOptions::default();

    assert_eq!(
        record.query(&query, &options).as_ref(),
        [["119232022", "abc", "119232022"]]
    );

    Ok(())
}
