use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::read_to_string;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_json_single_record() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("json").arg("tests/data/1004916019.dat").assert();

    let expected = read_to_string("tests/data/1004916019.json").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}

#[test]
fn pica_json_multiple_records() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("json")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/dump.json").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}

#[test]
fn pica_json_write_output() -> TestResult {
    let filename = Builder::new().suffix(".json").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("json")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/1004916019.dat")
        .assert();
    assert.success();

    let expected = read_to_string("tests/data/1004916019.json").unwrap();
    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(expected.trim_end().to_string(), actual);

    Ok(())
}

#[test]
fn pica_json_translit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("json")
        .arg("tests/data/004732650-reduced.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/004732650-nfd.json").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    let expected = vec![
        ("nfd", "tests/data/004732650-nfd.json"),
        ("nfkd", "tests/data/004732650-nfd.json"),
        ("nfc", "tests/data/004732650-nfc.json"),
        ("nfkc", "tests/data/004732650-nfc.json"),
    ];

    for (translit, output) in expected {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("json")
            .arg("--translit")
            .arg(translit)
            .arg("tests/data/004732650-reduced.dat.gz")
            .assert();

        let expected = read_to_string(output).unwrap();
        assert.success().stdout(expected.trim_end().to_string());
    }

    Ok(())
}

#[test]
fn pica_json_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("json")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::eq("[]"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("json").arg("tests/data/dump.dat.gz").assert();
    assert
        .failure()
        .code(1)
        .stderr(predicate::eq("Pica Error: Invalid record on line 2.\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[json]
skip-invalid = true
"#,
        )
        .arg("json")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.json").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("json")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.json").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[json]
skip-invalid = true
"#,
        )
        .arg("json")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.json").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[json]
skip-invalid = false
"#,
        )
        .arg("json")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.json").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}
