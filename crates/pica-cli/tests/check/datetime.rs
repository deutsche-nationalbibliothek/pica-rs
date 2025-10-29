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

    let mut cmd = pica_cmd();
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
            "ppn,rule,level,message\n123456789X,R1,error,2025-02-29\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
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
            path = "010@.D"
        "#,
        )
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e010@ \x1fDXYZ\x1fDABC\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R1,error,\"XYZ, ABC\"\n",
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

    let mut cmd = pica_cmd();
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
            "ppn,rule,level,message\n123456789X,R3,error,XYZ2025-02-29\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
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

    let mut cmd = pica_cmd();
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
            "ppn,rule,level,message\n123456789X,R4,error,25-02-29\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
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

#[test]
fn case_ignore() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let ruleset1 = temp_dir.child("rules1.toml");
    ruleset1
        .write_str(
            r#"
            [rule.R5]
            check = 'datetime'
            case-ignore = true
            path = '010@{ D | X == "foo" }'
        "#,
        )
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset1.to_str().unwrap()])
        .write_stdin(
            b"003@ \x1f0123456789X\x1e010@ \x1fD25-02-29\x1fXFoo\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "ppn,rule,level,message\n123456789X,R5,error,25-02-29\n",
        ))
        .stderr(predicates::str::is_empty());

    let ruleset2 = temp_dir.child("rules1.toml");
    ruleset2
        .write_str(
            r#"
            [rule.R6]
            check = 'datetime'
            case-ignore = false
            path = '010@{ D | X == "foo" }'
        "#,
        )
        .unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("check")
        .args(["-R", ruleset2.to_str().unwrap()])
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
