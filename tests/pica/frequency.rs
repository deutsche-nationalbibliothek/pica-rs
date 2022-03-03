use std::fs::read_to_string;

use assert_cmd::Command;
use predicates::prelude::predicate;
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
fn pica_frequency_multiple_files() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::eq("Tb1,8\nTp1,4\nTs1,2\n");
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_frequency_stdin() -> TestResult {
    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .write_stdin(data)
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("002@.0")
        .assert();

    let expected = predicate::eq("Ts1,1\n");
    assert.success().stdout(expected);

    let data = read_to_string("tests/data/000008672.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .write_stdin(data)
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .arg("-")
        .assert();

    let expected = predicate::eq("Tb1,5\nTp1,2\nTs1,1\n");
    assert.success().stdout(expected);

    let data = read_to_string("tests/data/000008672.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .write_stdin(data)
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("-")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::eq("Tb1,5\nTp1,2\nTs1,1\n");
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

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("abc")
        .arg("002@.0")
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
fn pica_frequency_translit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("029A.a")
        .arg("tests/data/004732650-reduced.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::eq(
            "Goethe-Universita\u{308}t Frankfurt am Main,1\n",
        ));

    let expected = vec![
        ("nfd", "Goethe-Universita\u{308}t Frankfurt am Main,1\n"),
        ("nfkd", "Goethe-Universita\u{308}t Frankfurt am Main,1\n"),
        ("nfc", "Goethe-Universität Frankfurt am Main,1\n"),
        ("nfkc", "Goethe-Universität Frankfurt am Main,1\n"),
    ];

    for (translit, output) in expected {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("frequency")
            .arg("--translit")
            .arg(translit)
            .arg("029A.a")
            .arg("tests/data/004732650-reduced.dat.gz")
            .assert();

        assert
            .success()
            .stderr(predicate::str::is_empty())
            .stdout(predicate::eq(output));
    }

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

#[test]
fn pica_frequency_invalid_path() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("frequency")
        .arg("--skip-invalid")
        .arg("002@.!")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stderr("Pica Error: Invalid path expression\n")
        .stdout(predicate::str::is_empty());

    Ok(())
}
