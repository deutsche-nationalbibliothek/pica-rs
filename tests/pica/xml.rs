use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::read_to_string;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_xml_single_record() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("xml").arg("tests/data/1004916019.dat").assert();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}

#[test]
fn pica_xml_multiple_records() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("xml")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/dump.xml").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}

#[test]
fn pica_xml_write_output() -> TestResult {
    let filename = Builder::new().suffix(".xml").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("xml")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/1004916019.dat")
        .assert();
    assert.success();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(expected.trim_end().to_string(), actual);

    Ok(())
}

#[test]
fn pica_xml_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("xml")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::eq("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<collection xmlns=\"info:srw/schema/5/picaXML-v1.0\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" targetNamespace=\"info:srw/schema/5/picaXML-v1.0\" />"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("xml").arg("tests/data/dump.dat.gz").assert();
    assert
        .failure()
        .code(1)
        .stderr(predicate::eq("Pica Error: Invalid record on line 2.\n"));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[xml]
skip-invalid = true
"#,
        )
        .arg("xml")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("xml")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[xml]
skip-invalid = true
"#,
        )
        .arg("xml")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[xml]
skip-invalid = false
"#,
        )
        .arg("xml")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}
