use assert_cmd::Command;
use predicates::prelude::*;

use crate::TestResult;

mod json;
mod print;
mod sample;
mod xml;

#[test]
fn pica_io_error() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("--config")
        .arg("/root/Pica.toml")
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("IO Error"));

    Ok(())
}
