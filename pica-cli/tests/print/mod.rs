use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use pica_record::matcher::{translit, NormalizationForm};

use crate::prelude::*;

#[test]
fn print_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert =
        cmd.arg("print").arg(data_dir().join("ada.dat")).assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_output() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .arg("print")
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let actual = read_to_string(out.path())?;
    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\n', "");
    }

    assert_eq!(expected, actual);

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn print_translit_nfc() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .args(["--translit", "nfc"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    let mut expected = translit(
        read_to_string(data_dir().join("algebra.txt"))?,
        Some(&NormalizationForm::Nfc),
    );

    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_translit_nfkc() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .args(["--translit", "nfkc"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    let mut expected = translit(
        read_to_string(data_dir().join("algebra.txt"))?,
        Some(&NormalizationForm::Nfkc),
    );

    if cfg!(windows) {
        expected = expected.replace('\n', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_translit_nfd() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .args(["--translit", "nfd"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    let mut expected = translit(
        read_to_string(data_dir().join("algebra.txt"))?,
        Some(&NormalizationForm::Nfd),
    );

    if cfg!(windows) {
        expected = expected.replace('\n', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_translit_nfkd() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("print")
        .args(["--translit", "nfkd"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    let mut expected = translit(
        read_to_string(data_dir().join("algebra.txt"))?,
        Some(&NormalizationForm::Nfkd),
    );

    if cfg!(windows) {
        expected = expected.replace('\n', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["print", "-s"])
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\n', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["print"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse erorr: invalid record on line 1",
        ));

    Ok(())
}
