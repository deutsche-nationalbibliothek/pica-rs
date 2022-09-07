use std::fs::read_to_string;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::Builder;

use crate::common::TestResult;

#[test]
fn pica_invalid_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert =
        cmd.arg("invalid").arg("tests/data/invalid.dat").assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/invalid.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert =
        cmd.arg("invalid").arg("tests/data/dump.dat.gz").assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/invalid.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_invalid_multiple_files() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("invalid")
        .arg("tests/data/004732650.dat.gz")
        .arg("tests/data/invalid.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/invalid.dat"));
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("invalid")
        .arg("tests/data/004732650.dat.gz")
        .arg("tests/data/000008672.dat")
        .arg("-")
        .write_stdin("foo\n")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("foo\n");

    Ok(())
}

#[test]
fn pica_invalid_stdin() -> TestResult {
    let data = read_to_string("tests/data/invalid.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("invalid").write_stdin(data).assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/invalid.dat"));
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let data = read_to_string("tests/data/invalid.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("invalid").arg("-").write_stdin(data).assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/invalid.dat"));
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    Ok(())
}

#[test]
fn pica_invalid_output() -> TestResult {
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    let expected = read_to_string("tests/data/invalid.dat").unwrap();
    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(expected, actual);

    Ok(())
}
