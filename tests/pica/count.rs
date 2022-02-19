use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::read_to_string;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_count_single_file() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/dump_cnt.txt")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
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
fn pica_count_multiple_files() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/000009229.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("records 2\nfields 55\nsubfields 114\n");

    Ok(())
}

#[test]
fn pica_count_stdin() -> TestResult {
    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.write_stdin(data).arg("count").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("records 1\nfields 22\nsubfields 43\n");

    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.write_stdin(data).arg("count").arg("-").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("records 1\nfields 22\nsubfields 43\n");

    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .write_stdin(data)
        .arg("count")
        .arg("tests/data/000009229.dat")
        .arg("-")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("records 2\nfields 55\nsubfields 114\n");

    Ok(())
}

#[test]
fn pica_count_tsv() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--tsv")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/dump_cnt.tsv")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
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
fn pica_count_csv() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--csv")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/dump_cnt.csv")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
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
fn pica_print_write_output() -> TestResult {
    let filename = Builder::new().suffix(".txt").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let expected = read_to_string("tests/data/dump_cnt.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_count_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    let expected = read_to_string("tests/data/invalid_cnt.txt")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("count").arg("tests/data/dump.dat.gz").assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::eq("Pica Error: Invalid record on line 2.\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[count]
    skip-invalid = true
    "#,
        )
        .arg("count")
        .arg("tests/data/invalid.dat")
        .assert();

    let expected = read_to_string("tests/data/invalid_cnt.txt")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
    skip-invalid = true
    "#,
        )
        .arg("count")
        .arg("tests/data/invalid.dat")
        .assert();

    let expected = read_to_string("tests/data/invalid_cnt.txt")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
    skip-invalid = false
    [count]
    skip-invalid = true
    "#,
        )
        .arg("count")
        .arg("tests/data/invalid.dat")
        .assert();

    let expected = read_to_string("tests/data/invalid_cnt.txt")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
    skip-invalid = false
    [count]
    skip-invalid = false
    "#,
        )
        .arg("count")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    let expected = read_to_string("tests/data/invalid_cnt.txt")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    Ok(())
}
