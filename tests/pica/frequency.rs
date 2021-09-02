use assert_cmd::Command;
use predicates::prelude::predicate;
use std::fs::read_to_string;
// use std::io::Read;
// use std::path::Path;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_frequency_default() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::eq("Tb1,4\nTp1,2\nTs1,1\n");
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_frequency_reverse() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--reverse")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::eq("Ts1,1\nTp1,2\nTb1,4\n");
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_frequency_limit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("2")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::eq("Tb1,4\nTp1,2\n");
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_frequency_multiple_subfields() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("1")
        .arg("047A/*.[erf]")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::eq("DE-1,8\n");
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_frequency_threshold() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--threshold")
        .arg("1")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(predicate::eq("Tb1,4\nTp1,2\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--threshold")
        .arg("2")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(predicate::eq("Tb1,4\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--threshold")
        .arg("3")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(predicate::eq("Tb1,4\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--threshold")
        .arg("999")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--threshold")
        .arg("abc")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::eq(
            "error: Invalid threshold value, expected unsigned integer.\n",
        ));

    Ok(())
}

#[test]
fn pica_frequency_empty_result() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("012A.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_frequency_header() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--header")
        .arg("bbg,cnt")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::eq("bbg,cnt\nTb1,4\nTp1,2\nTs1,1\n");
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--header")
        .arg("bbg,cnt,cnt2")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .stderr(predicate::str::starts_with("CSV Error:"));

    Ok(())
}

#[test]
fn pica_frequency_output() -> TestResult {
    let filename = Builder::new().suffix(".gz").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success();

    assert_eq!(read_to_string(filename).unwrap(), "Tb1,4\nTp1,2\nTs1,1\n");

    Ok(())
}

#[test]
fn pica_frequency_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stderr(predicate::eq("Pica Error: Invalid record on line 2.\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[frequency]
skip-invalid = true
"#,
        )
        .arg("frequency")
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("frequency")
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[frequency]
skip-invalid = true
"#,
        )
        .arg("frequency")
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[frequency]
skip-invalid = false
"#,
        )
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}
