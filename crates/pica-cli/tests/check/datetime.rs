use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn invalid() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R1]
            check = "datetime"
            path = "010@.D"
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e010@ \x1fD2025-02-29\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R1,error,\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e010@ \x1fD2024-02-29\x1e\n",
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

#[test]
fn message() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R1]
            check = "datetime"
            message = "invalid '{}'"
            path = "010@.D"
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e010@ \x1fDXYZ\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R1,error,invalid 'XYZ'\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn offset() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R3]
            check = "datetime"
            path = "010@.D"
            offset = 3
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e010@ \x1fDXYZ2025-02-29\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R3,error,\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e010@ \x1fDXYZ2024-02-29\x1e\n",
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

#[test]
fn format() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R4]
            check = "datetime"
            format = "%y-%m-%d"
            path = "010@.D"
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e010@ \x1fD25-02-29\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R4,error,\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e010@ \x1fD24-02-29\x1e\n",
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
