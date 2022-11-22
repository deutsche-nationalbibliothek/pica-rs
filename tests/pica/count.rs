use std::fs::read_to_string;

use assert_cmd::Command;
use predicates::prelude::*;
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
        .stdout("records: 2\nfields: 55\nsubfields: 114\n");

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
        .stdout("records: 1\nfields: 22\nsubfields: 43\n");

    let data = read_to_string("tests/data/1004916019.dat").unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.write_stdin(data).arg("count").arg("-").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("records: 1\nfields: 22\nsubfields: 43\n");

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
        .stdout("records: 2\nfields: 55\nsubfields: 114\n");

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
fn pica_print_append_output() -> TestResult {
    let filename = Builder::new().suffix(".csv").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--csv")
        .arg("--append")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/004732650.dat.gz")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--no-header")
        .arg("--csv")
        .arg("--append")
        .arg("--output")
        .arg(filename_str)
        .arg("tests/data/1004916019.dat.gz")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let expected = read_to_string(filename_str).unwrap();
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    assert_eq!(
        expected,
        "records,fields,subfields\n1,45,138\n1,22,43\n"
    );

    Ok(())
}

#[test]
fn pica_print_no_header() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--csv")
        .arg("--no-header")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("7,247,549\n");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--tsv")
        .arg("--no-header")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("7\t247\t549\n");

    Ok(())
}

#[test]
fn pica_print_single_value() -> TestResult {
    // --records
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--records")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("7\n");

    // --fields
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--fields")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("247\n");

    // --subfields
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("count")
        .arg("--skip-invalid")
        .arg("--subfields")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("549\n");

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
    let assert =
        cmd.arg("count").arg("tests/data/dump.dat.gz").assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "Parse Pica Error: invalid record",
        ));

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
