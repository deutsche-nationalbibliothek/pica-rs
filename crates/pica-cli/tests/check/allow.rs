use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn csv_list() -> TestResult {
    let temp_dir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.csv");
    allow
        .write_str("hsg,label\n004,Informatik\n010,Bibliografien\n")
        .unwrap();

    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(&format!(
            r#"
            [rule.R001]
            check = 'allow'
            list.filename = '{}'
            list.column = 'hsg'
            path = '045E.e'
        "#,
            allow.to_str().unwrap()
        ))
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e045E \x1fe200\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R001,error,200\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e045E \x1fe010\x1e\n")
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
fn tsv_list() -> TestResult {
    let temp_dir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.tsv");
    allow
        .write_str("hsg\tlabel\n004\tInformatik\n010\tBibliografien\n")
        .unwrap();

    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(&format!(
            r#"
            [rule.R001]
            check = 'allow'
            list.filename = '{}'
            list.column = 'hsg'
            path = '045E.e'
        "#,
            allow.to_str().unwrap()
        ))
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e045E \x1fe200\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R001,error,200\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(b"003@ \x1f0123456789X\x1e045E \x1fe010\x1e\n")
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

    let allow = temp_dir.child("ALLOW.csv");
    allow
        .write_str("hsg,label\n004,Informatik\n010,Bibliografien\n")
        .unwrap();

    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(&format!(
            r#"
            [rule.R001]
            check = 'allow'
            list.filename = '{}'
            list.column = 'label'
            path = '012A.a'
        "#,
            allow.to_str().unwrap()
        ))
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e012A \x1fainformatik\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R001,error,informatik\n",
        ))
        .stderr(predicates::str::is_empty());

    let ruleset = temp_dir.child("rules.toml");
    ruleset
        .write_str(&format!(
            r#"
            [rule.R001]
            check = 'allow'
            list.filename = '{}'
            list.column = 'label'
            case-ignore = true
            path = '012A.a'
        "#,
            allow.to_str().unwrap()
        ))
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e012A \x1faInformatik\x1e\n",
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
