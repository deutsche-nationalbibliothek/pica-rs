use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::PredicateBooleanExt;

use crate::prelude::*;

mod datetime;
mod duplicates;
mod filter;
mod isni;
mod iso639;
mod jel;
mod link;
mod unicode;

#[test]
fn check_skip_invalid() -> TestResult {
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
fn check_scope() -> TestResult {
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

#[test]
fn check_limit() -> TestResult {
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
        .args(["check", "--limit", "1"])
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("invalid.dat"))
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
fn check_where() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R001]
            check = "filter"
            filter = '004B.a != "pik"'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n040011569,R001,error,\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .args(["--where", "003@.0 == '119232022'"])
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
fn check_txt_output() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let output = temp_dir.child("out.txt");

    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R001]
            check = "filter"
            filter = '002@.0 =^ "Tp"'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["check", "-s"])
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", output.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(read_to_string(output)?, "118540238\n118607626\n");

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn check_tsv_output() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let output = temp_dir.child("out.tsv");

    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R001]
            check = "filter"
            filter = '002@.0 =^ "Tp"'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["check", "-s"])
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", output.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(output)?,
        "ppn\trule\tlevel\tmessage\n118540238\tR001\terror\t\n118607626\tR001\terror\t\n"
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn check_termination() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R001]
            check = "unicode"

            [rule.R002]
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
        .stdout(predicates::str::contains("123456789X,R001,error,\n"))
        .stdout(predicates::str::contains("123456789X,R002,error,\n"))
        .stderr(predicates::str::is_empty());

    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            termination = 'fast'
            
            [rule.R001]
            check = "unicode"

            [rule.R002]
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
        .stdout(
            predicates::ord::eq(
                "ppn,rule,level,message\n123456789X,R001,error,\n",
            )
            .or(predicates::ord::eq(
                "ppn,rule,level,message\n123456789X,R002,error,\n",
            )),
        )
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}
