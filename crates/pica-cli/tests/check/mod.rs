use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

mod datetime;
mod filter;
mod isni;
mod unicode;

#[test]
fn skip_invalid() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            scope = '003@.0 != "123456789X"'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["check", "-s"])
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn scope() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            scope = '003@.0 != "123456789X"'
            
            [rule.UNICODE]
            check = "unicode"
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e012A \x1f0\x00\x9F\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}
