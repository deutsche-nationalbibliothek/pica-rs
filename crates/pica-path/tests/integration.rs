use std::str::FromStr;
use std::sync::OnceLock;

use bstr::{ByteSlice, B};
use pica_matcher::MatcherOptions;
use pica_path::{ParsePathError, Path, PathExt};
use pica_record::ByteRecord;

type TestResult = anyhow::Result<()>;

fn ada_lovelace() -> &'static [u8] {
    use std::path::Path;
    use std::{env, fs};

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
fn path_new() -> TestResult {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    let options = MatcherOptions::default();
    let path = Path::new("003@.0");

    assert_eq!(record.path(&path, &options), vec!["119232022"]);
    Ok(())
}

#[test]
#[should_panic]
fn path_new_panic() {
    let _ = Path::new("003@.!");
}

#[test]
fn path_try_from() -> TestResult {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    let options = MatcherOptions::default();
    let path = Path::try_from(B("003@.0"))?;

    assert_eq!(record.path(&path, &options), vec!["119232022"]);
    assert!(matches!(
        Path::try_from(B("003@.!")).unwrap_err(),
        ParsePathError(_)
    ));

    Ok(())
}

#[test]
fn path_from_str() -> TestResult {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    let options = MatcherOptions::default();
    let path = Path::from_str("003@.0")?;

    assert_eq!(record.path(&path, &options), vec!["119232022"]);
    assert!(matches!(
        Path::from_str("003@.!").unwrap_err(),
        ParsePathError(_)
    ));

    Ok(())
}

#[test]
fn path_codes() {
    assert_eq!(Path::new("003@.0").codes(), &[vec!['0']]);
    assert_eq!(
        Path::new("003@{ [01], 2 }").codes(),
        &[vec!['0', '1'], vec!['2']]
    );

    assert_eq!(
        Path::new("003@{ ([0-2], 2) }").codes(),
        &[vec!['0', '1', '2'], vec!['2']]
    );
}

#[test]
fn path_codes_flat() {
    assert_eq!(Path::new("003@.0").codes_flat(), &['0']);
    assert_eq!(
        Path::new("003@{ [01], 2 }").codes_flat(),
        &['0', '1', '2']
    );

    assert_eq!(
        Path::new("003@{ ([0-2], 2) }").codes_flat(),
        &['0', '1', '2', '2']
    );
}

#[test]
fn path_simple() -> TestResult {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    let options = MatcherOptions::default();
    let path = Path::new("003@.0");

    assert_eq!(record.path(&path, &options), vec!["119232022"]);
    Ok(())
}

#[test]
fn path_matcher() -> TestResult {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    let path = Path::new("065R{ 9 | 4 == 'ortg' }");
    let options = MatcherOptions::default();

    assert_eq!(record.path(&path, &options), vec!["040743357"]);
    Ok(())
}

#[test]
fn path_idn() -> TestResult {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    assert_eq!(record.idn(), Some(b"119232022".as_bstr()));
    Ok(())
}

#[test]
fn test_path_codes() -> TestResult {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    let options = MatcherOptions::default();
    let path = Path::new("047A/03.[er]");

    assert_eq!(record.path(&path, &options), vec!["DE-386", "DE-576"]);
    Ok(())
}
