use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn check_duplicates_path() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R001]
            check = 'duplicates'
            query = '045Z{ a | b == "jelc" }'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e045Z \x1fbjelc\x1faD63\x1faD63\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R001,error,D63\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e045Z \x1fbjelc\x1faD63\x1e\n",
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
fn check_duplicates_query() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R001]
            check = 'duplicates'
            query = '045E{ E, H }'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e045E \x1fe100\x1fEi\x1fHdnb\x1e045E \x1fe110\x1fEi\x1fHdnb\x1e\n", )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R001,error,i-dnb\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e045E \x1fe100\x1fEm\x1fHdnb\x1e045E \x1fe110\x1fEi\x1fHdnb\x1e\n", )
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
fn check_duplicates_threshold() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R001]
            check = 'duplicates'
            threshold = 3
            query = '045E{ E, H }'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e045E \x1fe100\x1fEi\x1fHdnb\x1e045E \x1fe110\x1fEi\x1fHdnb\x1e\n", )
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
        .write_stdin(
            b"003@ \x1f0123456789X\x1e045E \x1fe100\x1fEi\x1fHdnb\x1e045E \x1fe110\x1fEi\x1fHdnb\x1e045E \x1fe200\x1fEi\x1fHdnb\x1e\n", )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R001,error,i-dnb\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn check_duplicates_ignore_case() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R001]
            check = 'duplicates'
            case-ignore = true
            query = '045Z{ a | b =^ "JEL" }'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e045Z \x1fbjelc\x1faD63\x1faD63\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R001,error,D63\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}
