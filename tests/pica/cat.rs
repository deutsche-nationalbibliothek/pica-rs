use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_cat_missing_file() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/10049160XX.dat")
        .assert();

    assert
        .failure()
        .stderr(predicate::str::starts_with("IO Error:"))
        .stdout(predicate::str::is_empty())
        .code(1);
    Ok(())
}
