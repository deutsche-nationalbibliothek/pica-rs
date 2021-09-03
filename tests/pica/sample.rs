use assert_cmd::Command;
use flate2::read::GzDecoder;
use predicates::prelude::*;
use std::fs::{read_to_string, File};
use std::io::Read;
use std::path::Path;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_cat_sample() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("1")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let data_dir = Path::new("tests/data");

    assert.success().stdout(
        predicate::never()
            .or(predicate::path::eq_file(data_dir.join("1004916019.dat")))
            .or(predicate::path::eq_file(data_dir.join("119232022.dat")))
            .or(predicate::path::eq_file(data_dir.join("000008672.dat")))
            .or(predicate::path::eq_file(data_dir.join("000016586.dat")))
            .or(predicate::path::eq_file(data_dir.join("000016756.dat")))
            .or(predicate::path::eq_file(data_dir.join("000009229.dat")))
            .or(predicate::path::eq_file(data_dir.join("121169502.dat"))),
    );

    Ok(())
}

#[test]
fn pica_sample_size_le_len() -> TestResult {
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("2")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success();

    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(actual.lines().count(), 2);
    Ok(())
}

#[test]
fn pica_sample_size_eq_len() -> TestResult {
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("7")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success();

    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(actual.lines().count(), 7);
    Ok(())
}

#[test]
fn pica_sample_size_gt_len() -> TestResult {
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("8")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success();

    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(actual.lines().count(), 7);
    Ok(())
}

#[test]
fn pica_sample_size_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("--skip-invalid")
        .arg("0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr("error: invalid sample size \'0\'. expected non-zero usize.\n");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("--skip-invalid")
        .arg("a")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr("error: invalid sample size \'a\'. expected non-zero usize.\n");

    Ok(())
}

#[test]
fn pica_sample_write_output() -> TestResult {
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("1")
        .arg("tests/data/1004916019.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
    assert!(expected.eval(Path::new(filename_str)));

    Ok(())
}

#[test]
fn pica_sample_write_gzip() -> TestResult {
    let expected = read_to_string("tests/data/1004916019.dat").unwrap();

    // filename
    let filename = Builder::new().suffix(".dat.gz").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("1")
        .arg("tests/data/1004916019.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut gz = GzDecoder::new(File::open(filename_str).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();
    assert_eq!(expected, actual);

    // flag
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("--skip-invalid")
        .arg("--gzip")
        .arg("--output")
        .arg(filename_str)
        .arg("1")
        .arg("tests/data/1004916019.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut gz = GzDecoder::new(File::open(filename_str).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();
    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_sample_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("1")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("sample")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::eq("Pica Error: Invalid record on line 2.\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("sample")
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[sample]
skip-invalid = true
"#,
        )
        .arg("sample")
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[sample]
skip-invalid = true
"#,
        )
        .arg("sample")
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[sample]
skip-invalid = false
"#,
        )
        .arg("sample")
        .arg("--skip-invalid")
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}
