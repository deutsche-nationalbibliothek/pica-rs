use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

// use predicates::prelude::*;
use crate::prelude::*;

#[test]
fn simple() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R01]
            check = 'link'
            source = '065R.9'
            destination = '003@.0'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["check", "-s"])
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains(
            "118607626,R01,error,\"040374432, 04028557X, 04037372X\"\n",
        ))
        .stdout(predicates::str::contains(
            "118540238,R01,error,040181189\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn path_filter() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R01]
            check = 'link'
            source = '065R{ 9 | 4 == "ortw" }'
            destination = '003@.0'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["check", "-s"])
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains(
            "118607626,R01,error,\"04028557X, 04037372X\"\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn condition() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(
            r#"
            [rule.R01]
            check = 'link'
            source = '065R{ 9 | 4 == "ortw" }'
            destination = '003@.0'
            condition = '002@.0 == "Tgz"'
        "#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["check", "-s"])
        .args(["-R", ruleset.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains(
            "118607626,R01,error,\"04028557X, 04037372X\"\n",
        ))
        .stdout(predicates::str::contains(
            "118540238,R01,error,040651053\n",
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}
