use crate::support::{CommandBuilder, MatchResult, SAMPLE1, SAMPLE2};
use std::fs::read_to_string;
use tempfile::Builder;

#[test]
fn cat_no_file() -> MatchResult {
    CommandBuilder::new("cat").with_status(2).run()?;
    Ok(())
}

#[test]
fn cat_single_file() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/1004916019.dat")
        .with_stdout(SAMPLE1)
        .run()?;

    Ok(())
}

#[test]
fn cat_multiple_files() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn cat_gzip_file() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/119232022.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn cat_missing_file() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/123456789X.dat")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn cat_invalid_file() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn cat_skip_invalid() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/invalid.dat")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn cat_write_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-cat").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("cat")
        .arg("--skip-invalid")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/119232022.dat")
        .with_stdout_empty()
        .run()?;

    assert_eq!(read_to_string(filename).unwrap(), SAMPLE2);
    Ok(())
}
