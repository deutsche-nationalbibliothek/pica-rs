use crate::support::{CommandBuilder, MatchResult, INVALID};
use std::fs::read_to_string;
use tempfile::Builder;

#[test]
fn invalid_single_record() -> MatchResult {
    CommandBuilder::new("invalid")
        .arg("tests/data/invalid.dat")
        .with_stdout(INVALID)
        .run()?;

    Ok(())
}

#[test]
fn invalid_multiple_record() -> MatchResult {
    CommandBuilder::new("invalid")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(INVALID)
        .run()?;

    Ok(())
}

#[test]
fn invalid_write_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-cat").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("invalid")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    assert_eq!(read_to_string(filename).unwrap(), INVALID);
    Ok(())
}
