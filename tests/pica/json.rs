use std::fs::read_to_string;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

const DEPRICATION_WARNING: &str = "WARNING: The `json` command will be removed in version 0.17, please use the `convert` command instead.";

#[test]
fn pica_json_single_record() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert =
        cmd.arg("json").arg("tests/data/1004916019.dat").assert();

    let expected =
        read_to_string("tests/data/1004916019.json").unwrap();
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

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("json")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/000008672.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/tworecs.json"));
    assert
        .success()
        .stderr(predicate::str::starts_with(DEPRICATION_WARNING))
        // .stderr(predicate::str::is_empty())
        .stdout(expected);

    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("json")
        .arg("-")
        .arg("tests/data/000008672.dat")
        .write_stdin(data)
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/tworecs.json"));
    assert
        .success()
        .stderr(predicate::str::starts_with(DEPRICATION_WARNING))
        .stdout(expected);

    Ok(())
}

#[test]
fn pica_json_stdin() -> TestResult {
    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("json").arg("-").write_stdin(data).assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.json",
    ));

    assert
        .success()
        .stderr(predicate::str::starts_with(DEPRICATION_WARNING))
        .stdout(expected);

    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("json").write_stdin(data).assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.json",
    ));

    assert
        .success()
        .stderr(predicate::str::starts_with(DEPRICATION_WARNING))
        .stdout(expected);

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

    let expected =
        read_to_string("tests/data/1004916019.json").unwrap();
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

    let expected =
        read_to_string("tests/data/004732650-nfd.json").unwrap();
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
        .stderr(predicate::str::starts_with(DEPRICATION_WARNING))
        .stderr(predicate::str::contains(
            "Pica Error: Invalid record on line 2.\n",
        ));

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

    let expected =
        read_to_string("tests/data/1004916019.json").unwrap();
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

    let expected =
        read_to_string("tests/data/1004916019.json").unwrap();
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

    let expected =
        read_to_string("tests/data/1004916019.json").unwrap();
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

    let expected =
        read_to_string("tests/data/1004916019.json").unwrap();
    assert.success().stdout(expected.trim_end().to_string());

    Ok(())
}
