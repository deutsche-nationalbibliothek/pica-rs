use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::read_to_string;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_print_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("print").arg("tests/data/1004916019.dat").assert();

    let expected = read_to_string("tests/data/1004916019.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace("\r", "")
    } else {
        expected
    };

    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_print_multiple_records() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/dump.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace("\r", "")
    } else {
        expected
    };

    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_print_limit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/1004916019.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace("\r", "")
    } else {
        expected
    };

    assert.success().stdout(expected);

    // invalid limit
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("abc")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::eq(
            "error: Invalid limit value, expected unsigned integer.\n",
        ));

    Ok(())
}

#[test]
fn pica_print_color() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("--color")
        .arg("always")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019-color.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace("\r", "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("--color")
        .arg("never")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace("\r", "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    Ok(())
}

#[test]
fn pica_print_add_spaces() -> TestResult {
    let expected = read_to_string("tests/data/1004916019-spaces.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace("\r", "")
    } else {
        expected
    };

    // CLI flag
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("--add-spaces")
        .arg("tests/data/1004916019.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected.to_owned());

    // Config
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[print]
add-spaces = true
"#,
        )
        .arg("print")
        .arg("tests/data/1004916019.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    Ok(())
}

#[test]
fn pica_print_write_output() -> TestResult {
    let filename = Builder::new().suffix(".txt").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/1004916019.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let expected = read_to_string("tests/data/1004916019.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace("\r", "")
    } else {
        expected
    };

    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_print_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("print").arg("tests/data/invalid.dat").assert();
    assert
        .failure()
        .stderr(predicate::eq("Pica Error: Invalid record on line 1.\n"))
        .stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("print")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[print]
skip-invalid = true
"#,
        )
        .arg("print")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[print]
skip-invalid = true
"#,
        )
        .arg("print")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[print]
skip-invalid = false
"#,
        )
        .arg("print")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}
