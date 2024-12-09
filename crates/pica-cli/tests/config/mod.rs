use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn set_option() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let config = temp_dir.child("pica.toml");
    let filename = config.to_str().unwrap();

    let assert = cmd
        .args(["--config", filename])
        .arg("config")
        .args(["skip-invalid", "true"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::str::contains("skip-invalid = true")
        .eval(&read_to_string(config).unwrap()));

    temp_dir.close().unwrap();

    Ok(())
}

#[test]
fn get_option() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let config = temp_dir.child("pica.toml");
    let filename = config.to_str().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["--config", filename])
        .args(["config", "skip-invalid"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains("false"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["--config", filename])
        .arg("config")
        .args(["skip-invalid", "true"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["--config", filename])
        .args(["config", "skip-invalid"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains("true"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    Ok(())
}

#[test]
fn unset_option() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let config = temp_dir.child("pica.toml");
    let filename = config.to_str().unwrap();

    // get (option is not set yet)
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["--config", filename])
        .args(["config", "normalization"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains("None"))
        .stderr(predicates::str::is_empty());

    // set option
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["--config", filename])
        .arg("config")
        .args(["normalization", "nfd"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["--config", filename])
        .args(["config", "normalization"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains("nfd"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["--config", filename])
        .args(["config", "--unset", "normalization"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["--config", filename])
        .args(["config", "--get", "normalization"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains("None"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    Ok(())
}

#[test]
fn invalid_option() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let config = temp_dir.child("pica.toml");
    let filename = config.to_str().unwrap();

    let assert = cmd
        .args(["--config", filename])
        .arg("config")
        .args(["foobar", "true"])
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "error: unknown config option `foobar`\n",
        ));

    assert!(predicates::path::missing().eval(&config));
    temp_dir.close().unwrap();

    Ok(())
}
