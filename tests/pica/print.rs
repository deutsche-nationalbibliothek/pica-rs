use crate::support::{CommandBuilder, MatchResult};
use std::fs::read_to_string;
use tempfile::Builder;

#[test]
fn print_single_record() -> MatchResult {
    let exptected = read_to_string("tests/data/1004916019.txt").unwrap();
    let exptected = if cfg!(target_os = "windows") {
        exptected.replace("\n", "\r\n")
    } else {
        exptected
    };

    CommandBuilder::new("print")
        .arg("tests/data/1004916019.dat")
        .with_stdout(&exptected)
        .run()?;

    Ok(())
}

#[test]
fn print_multiple_records() -> MatchResult {
    let exptected = read_to_string("tests/data/dump.txt").unwrap();
    let exptected = if cfg!(target_os = "windows") {
        exptected.replace("\n", "\r\n")
    } else {
        exptected
    };

    CommandBuilder::new("print")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(&exptected)
        .run()?;

    Ok(())
}

#[test]
fn print_gzip_file() -> MatchResult {
    let exptected = read_to_string("tests/data/1004916019.txt").unwrap();
    let exptected = if cfg!(target_os = "windows") {
        exptected.replace("\n", "\r\n")
    } else {
        exptected
    };

    CommandBuilder::new("print")
        .arg("tests/data/1004916019.dat.gz")
        .with_stdout(&exptected)
        .run()?;

    Ok(())
}

#[test]
fn print_write_output() -> MatchResult {
    let exptected = read_to_string("tests/data/1004916019.txt").unwrap();
    let exptected = if cfg!(target_os = "windows") {
        exptected.replace("\n", "\r\n")
    } else {
        exptected
    };

    let tempdir = Builder::new().prefix("pica-print").tempdir().unwrap();
    let filename = tempdir.path().join("sample.txt");

    CommandBuilder::new("print")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    assert_eq!(exptected, read_to_string(filename).unwrap());

    Ok(())
}

#[test]
fn print_invalid_file() -> MatchResult {
    CommandBuilder::new("print")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
