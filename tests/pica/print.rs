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
        expected.replace('\r', "")
    } else {
        expected
    };

    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_print_escape_dollar() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("print").arg("tests/data/dollar.dat").assert();

    let expected = read_to_string("tests/data/dollar.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
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
        expected.replace('\r', "")
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
        expected.replace('\r', "")
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

    let expected = read_to_string("tests/data/1004916019-color1.txt").unwrap();
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
            r#"[print]
    field-color = { color = "red", bold = true, intense = true }
    occurrence-color = { color = "blue", underline = true }
    code-color = { color = "165,42,42", italic = false }
    value-color = { color = "95", dimmed = true }
    "#,
        )
        .arg("print")
        .arg("--color")
        .arg("always")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019-color2.txt").unwrap();
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
        .arg("print")
        .arg("--color")
        .arg("never")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.txt").unwrap();
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
fn pica_print_add_spaces() -> TestResult {
    let expected = read_to_string("tests/data/1004916019-spaces.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
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
        expected.replace('\r', "")
    } else {
        expected
    };

    let actual = read_to_string(filename_str).unwrap();
    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_print_translit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .arg("tests/data/004732650-reduced.dat.gz")
        .assert();

    let expected = read_to_string("tests/data/004732650-nfd.txt").unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert.success().stdout(expected.to_string());

    let expected = vec![
        ("nfd", "tests/data/004732650-nfd.txt"),
        ("nfkd", "tests/data/004732650-nfd.txt"),
        ("nfc", "tests/data/004732650-nfc.txt"),
        ("nfkc", "tests/data/004732650-nfc.txt"),
    ];

    for (translit, output) in expected {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("print")
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

        assert.success().stdout(expected.to_string());
    }

    let expected = vec![
        ("nfd", "029A $aGoethe-Universita\u{308}t"),
        ("nfkd", "029A $aGoethe-Universita\u{308}t"),
        ("nfc", "029A $aGoethe-Universität"),
        ("nfkc", "029A $aGoethe-Universität"),
    ];

    for (translit, prefix) in expected {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("print")
            .arg("--translit")
            .arg(translit)
            .arg("tests/data/004732650-reduced.dat.gz")
            .assert();

        assert.success().stdout(predicate::str::starts_with(prefix));
    }
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
