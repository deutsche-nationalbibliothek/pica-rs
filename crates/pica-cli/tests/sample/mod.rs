// use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn sample_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "1"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn sample_output() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "1"])
        .arg(data_dir().join("ada.dat"))
        .args(["--output", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::eq_file(data_dir().join("ada.dat"))
        .eval(samples.path()));

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_gzip() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat.gz");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "2"])
        .arg(data_dir().join("ada.dat"))
        .args(["--output", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(samples.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "10"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty().not())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "10"])
        .arg(data_dir().join("DUMP.dat.gz"))
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

#[test]
fn sample_seed() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat.gz");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "2"])
        .args(["--seed", "1234"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::eq_file(
        data_dir().join("samples.dat.gz")
    )
    .eval(samples.path()));

    temp_dir.close().unwrap();
    Ok(())
}