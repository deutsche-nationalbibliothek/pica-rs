use assert_cmd::Command;
use flate2::read::GzDecoder;
use predicates::prelude::*;
use std::fs::{read_to_string, File};
use std::io::Read;
use std::path::Path;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_invalid_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("invalid").arg("tests/data/invalid.dat").assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/invalid.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("invalid").arg("tests/data/dump.dat.gz").assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/invalid.dat"));
    assert.success().stdout(expected);

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
