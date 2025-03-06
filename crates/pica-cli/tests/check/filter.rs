use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

// use predicates::prelude::*;
use crate::prelude::*;

#[test]
fn invalid() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R1]
            check = "filter"
            filter = '017C.a not in ["a","d","f","i","m","n","t"]'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e017C \x1fax\x1e\n")
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
        .write_stdin(b"003@ \x1f0123456789X\x1e017C \x1faf\x1e\n")
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
fn invert_match() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R2]
            check = "filter"
            filter = '012A.a == "abc"'
            invert-match = true
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e012A \x1fadef\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R2,error,\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e012A \x1faabc\x1e\n")
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
fn case_ignore() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R3]
            check = "filter"
            filter = '012A.a == "abc"'
            case-ignore = true
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e012A \x1faABC\x1e\n")
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
        .write_stdin(b"003@ \x1f0123456789X\x1e012A \x1faabc\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R3,error,\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R4]
            check = "filter"
            filter = '012A.a == "abc"'
            case-ignore = false
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e012A \x1faABC\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e012A \x1faabc\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R4,error,\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}
