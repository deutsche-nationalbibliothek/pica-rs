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
            [rule.UNICODE]
            check = "unicode"
        "#,
        )
        .unwrap();

    let mut cmd = pica_cmd();
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
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,UNICODE,error,\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    Ok(())
}

#[test]
fn normalization() -> TestResult {
    // RULE nfd <-> DATA nfc
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.UNICODE]
            check = "unicode"
            normalization = "nfd"
        "#,
        )
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e012A \x1f0\xc3\xa4\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,UNICODE,error,ä\n",
        ))
        .stderr(predicates::str::is_empty());

    // RULE nfd <-> DATA nfd
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.UNICODE]
            check = "unicode"
            normalization = "nfd"
        "#,
        )
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1F0123456789X\x1E012A \x1F0\x61\xCC\x88\x1E\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    Ok(())
}
