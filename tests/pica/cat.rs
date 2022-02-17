use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::read_to_string;
use std::path::Path;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_cat_single_file() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_cat_multiple_files() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/000009229.dat")
        .assert();

    let expected = format!(
        "{}{}",
        read_to_string("tests/data/1004916019.dat").unwrap(),
        read_to_string("tests/data/000009229.dat").unwrap()
    );
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_cat_stdin() -> TestResult {
    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("cat").write_stdin(data).assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("cat").arg("-").write_stdin(data).assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("-")
        .arg("tests/data/1004916019.dat")
        .write_stdin("foo")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);
    Ok(())
}

#[test]
fn pica_cat_read_gzip() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat.gz")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_cat_write_gzip() -> TestResult {
    let filename = Builder::new().suffix(".gz").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--gzip")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/1004916019.dat")
        .assert();
    assert.success();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("cat").arg(filename_str).assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_cat_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("cat").arg("tests/data/dump.dat.gz").assert();
    assert
        .failure()
        .code(1)
        .stdout(predicate::path::eq_file(Path::new(
            "tests/data/1004916019.dat",
        )))
        .stderr(predicate::eq("Pica Error: Invalid record on line 2.\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[cat]
skip-invalid = true
"#,
        )
        .arg("cat")
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
        .arg("cat")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[cat]
skip-invalid = true
"#,
        )
        .arg("cat")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[cat]
skip-invalid = false
"#,
        )
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_cat_missing_file() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/10049160XX.dat")
        .assert();

    assert
        .failure()
        .stderr(predicate::str::starts_with("Pica Error:"))
        .stdout(predicate::str::is_empty())
        .code(1);
    Ok(())
}
