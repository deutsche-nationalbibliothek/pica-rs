use std::fs::read_to_string;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_xml_single_record() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert =
        cmd.arg("xml").arg("tests/data/1004916019.dat").assert();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

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
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}

#[test]
fn pica_xml_multiple_files() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("xml")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/000008672.dat")
        .assert();

    let expected =
        read_to_string("tests/data/two-records.xml").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected.trim_end().to_string());

    let data = read_to_string("tests/data/1004916019.dat")?;
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("xml")
        .arg("-")
        .arg("tests/data/000008672.dat")
        .write_stdin(data)
        .assert();

    let expected =
        read_to_string("tests/data/two-records.xml").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected.trim_end().to_string());

    Ok(())
}

#[test]
fn pica_xml_stdin() -> TestResult {
    let data = read_to_string("tests/data/1004916019.dat")?;
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("xml").write_stdin(data).assert();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected.trim_end().to_string());

    let data = read_to_string("tests/data/1004916019.dat")?;
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("xml").arg("-").write_stdin(data).assert();

    let expected = read_to_string("tests/data/1004916019.xml").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected.trim_end().to_string());

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
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(expected.trim_end().to_string(), actual);

    Ok(())
}

#[test]
fn pica_xml_translit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("xml")
        .arg("tests/data/004732650-reduced.dat.gz")
        .assert();

    let expected =
        read_to_string("tests/data/004732650-nfd.xml").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert.success().stdout(expected.trim_end().to_string());

    let expected = vec![
        ("nfd", "tests/data/004732650-nfd.xml"),
        ("nfkd", "tests/data/004732650-nfd.xml"),
        ("nfc", "tests/data/004732650-nfc.xml"),
        ("nfkc", "tests/data/004732650-nfc.xml"),
    ];

    for (translit, output) in expected {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("xml")
            .arg("--translit")
            .arg(translit)
            .arg("tests/data/004732650-reduced.dat.gz")
            .assert();

        let expected = read_to_string(output).unwrap();
        let expected = if cfg!(windows) {
            expected.replace('\r', "")
        } else {
            expected
        };

        assert.success().stdout(expected.trim_end().to_string());
    }

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

    let expected = read_to_string("tests/data/invalid.xml").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert.success().stdout(expected.trim_end().to_string());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("xml").arg("tests/data/dump.dat.gz").assert();
    assert.failure().code(1).stderr(predicate::eq(
        "Pica Error: Invalid record on line 2.\n",
    ));

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
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

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
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

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
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

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
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}
