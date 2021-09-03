use assert_cmd::Command;
use flate2::read::GzDecoder;
use predicates::prelude::*;
use std::fs::{read_to_string, File};
use std::io::Read;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_slice_default() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    Ok(())
}

#[test]
fn pica_slice_write_output() -> TestResult {
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    let actual = read_to_string(filename_str)?;

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_slice_write_gzip() -> TestResult {
    // filename
    let filename = Builder::new().suffix(".dat.gz").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--end")
        .arg("1")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    let expected = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut gz = GzDecoder::new(File::open(filename_str).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();
    assert_eq!(expected, actual);

    // flag
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--gzip")
        .arg("--end")
        .arg("1")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    let expected = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut gz = GzDecoder::new(File::open(filename_str).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();
    assert_eq!(expected, actual);

    // config
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[slice]
gzip = true
"#,
        )
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--end")
        .arg("1")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    let expected = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut gz = GzDecoder::new(File::open(filename_str).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();
    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_slice_start() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--start")
        .arg("0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--start")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--start")
        .arg("999")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--start")
        .arg("abc")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr("error: invalid start option\n");

    Ok(())
}

#[test]
fn pica_slice_end() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--end")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/1004916019.dat").unwrap();
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    // invalid record on position 1
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--end")
        .arg("2")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/1004916019.dat").unwrap();
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--end")
        .arg("3")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--end")
        .arg("999")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    // invalid end option
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--end")
        .arg("abc")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr("error: invalid end option\n");

    Ok(())
}

#[test]
fn pica_slice_length() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--length")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/1004916019.dat").unwrap();
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--length")
        .arg("2")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--length")
        .arg("100")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--start")
        .arg("1")
        .arg("--length")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/119232022.dat").unwrap();
    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    // invalid length option
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("--length")
        .arg("abc")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr("error: invalid length option\n");

    Ok(())
}

#[test]
fn pica_slice_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("slice")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("slice").arg("tests/data/invalid.dat").assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr("Pica Error: Invalid record on line 1.\n");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("slice")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[slice]
skip-invalid = true
"#,
        )
        .arg("slice")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[slice]
skip-invalid = true

"#,
        )
        .arg("slice")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[slice]
skip-invalid = false

"#,
        )
        .arg("slice")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    Ok(())
}
