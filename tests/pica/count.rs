use assert_cmd::Command;
use flate2::read::GzDecoder;
use predicates::prelude::*;
use std::fs::{read_to_string, File};
use std::io::Read;
use std::path::Path;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_count() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/dump_cnt.txt"));
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--tsv")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/dump_cnt.tsv"));
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--csv")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/dump_cnt.csv"));
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    Ok(())
}

#[test]
fn pica_count_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/invalid_cnt.txt"));
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

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    Ok(())
}
