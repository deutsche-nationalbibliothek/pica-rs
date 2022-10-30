use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_cat_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("cat").arg("tests/data/dump.dat.gz").assert();
    assert
        .failure()
        .code(1)
        .stdout(predicate::path::eq_file(Path::new(
            "tests/data/1004916019.dat",
        )))
        .stderr(predicate::str::starts_with(
            "Parse Pica Error: invalid record",
        ));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[cat]
skip-invalid = true
"#,
        )
        .arg("cat")
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
        .arg("cat")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[cat]
skip-invalid = true
"#,
        )
        .arg("cat")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[cat]
skip-invalid = false
"#,
        )
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

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
