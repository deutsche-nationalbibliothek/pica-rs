use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::Builder;

use crate::common::TestResult;

#[test]
fn pica_bash_completion() -> TestResult {
    let filename = Builder::new().tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("completions")
        .arg("bash")
        .arg("--output")
        .arg(filename_str)
        .assert();
    assert.success();
    assert!(predicates::path::is_file().eval(filename_str));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("completions").arg("bash").assert();
    assert
        .success()
        .stdout(predicate::path::eq_file(filename_str));

    Ok(())
}

#[test]
fn pica_fish_completion() -> TestResult {
    let filename = Builder::new().tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("completions")
        .arg("fish")
        .arg("--output")
        .arg(filename_str)
        .assert();
    assert.success();
    assert!(predicates::path::is_file().eval(filename_str));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("completions").arg("fish").assert();
    assert
        .success()
        .stdout(predicate::path::eq_file(filename_str));

    Ok(())
}

#[test]
fn pica_zsh_completion() -> TestResult {
    let filename = Builder::new().tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("completions")
        .arg("zsh")
        .arg("--output")
        .arg(filename_str)
        .assert();
    assert.success();
    assert!(predicates::path::is_file().eval(filename_str));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("completions").arg("zsh").assert();
    assert
        .success()
        .stdout(predicate::path::eq_file(filename_str));

    Ok(())
}
