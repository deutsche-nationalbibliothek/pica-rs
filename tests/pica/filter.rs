use std::fs::read_to_string;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_filter_limit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("1")
        .arg("003@.0?")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("99")
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("0")
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--limit")
        .arg("abc")
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/dump.dat.gz")
        .assert();

    // error code "2" is set by clap-rs
    assert.failure().code(2).stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_ignore_case() -> TestResult {
    // `==` Operator
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("050E.a == 'internet'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--ignore-case")
        .arg("050E.a == 'internet'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    // `!=` Operator
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("050E.a != 'internet'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--ignore-case")
        .arg("050E.a != 'internet'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    // `=^` Operator
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("050E.a =^ 'inter'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--ignore-case")
        .arg("050E.a =^ 'inter'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    // `=$` Operator
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("050E.a =$ 'neT'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--ignore-case")
        .arg("050E.a =$ 'neT'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    // `=~` Operator
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("050E.a =~ '^internet'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--ignore-case")
        .arg("050E.a =~ '^internet'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    // `in` Operator
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("050E.a in ['internet', 'inTernet']")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--ignore-case")
        .arg("050E.a in ['internet', 'inTernet']")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_filter_expression_file() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--file")
        .arg("tests/data/filter.txt")
        .arg("True")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/119232022.dat"));
    assert.success().stdout(expected);

    // invalid expression file
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--file")
        .arg("tests/data/119232022.dat")
        .arg("True")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: invalid filter: "));

    Ok(())
}

#[test]
fn pica_filter_tee_option() -> TestResult {
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--tee")
        .arg(filename_str)
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert!(expected.eval(Path::new(filename_str)));

    Ok(())
}

#[test]
fn pica_filter_append_option() -> TestResult {
    // --output
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/1004916019.dat")
        .arg("--output")
        .arg(filename_str)
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--append")
        .arg("003@.0 == '000009229'")
        .arg("tests/data/000009229.dat")
        .arg("--output")
        .arg(filename_str)
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = format!(
        "{}{}",
        read_to_string("tests/data/1004916019.dat").unwrap(),
        read_to_string("tests/data/000009229.dat").unwrap()
    );

    assert_eq!(expected, read_to_string(filename_str)?);

    // --tee
    let filename = Builder::new().suffix(".dat").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("003@.0 == '1004916019'")
        .arg("--tee")
        .arg(filename_str)
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--append")
        .arg("--tee")
        .arg(filename_str)
        .arg("003@.0 == '000009229'")
        .arg("tests/data/000009229.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/000009229.dat"));

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(expected);

    let expected = format!(
        "{}{}",
        read_to_string("tests/data/1004916019.dat").unwrap(),
        read_to_string("tests/data/000009229.dat").unwrap()
    );

    assert_eq!(expected, read_to_string(filename_str)?);

    Ok(())
}

#[test]
fn pica_filter_invalid_filter() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.!?")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::eq("error: invalid filter: \"003@.!?\"\n"));

    Ok(())
}

#[test]
fn pica_filter_missing_file() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0?")
        .arg("tests/data/dump2.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("IO Error: "));

    Ok(())
}

#[test]
fn pica_filter_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '121169502'")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("003@.0?")
        .arg("tests/data/dump.dat.gz")
        .assert();

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
            r#"[filter]
skip-invalid = true
"#,
        )
        .arg("filter")
        .arg("003@.0?")
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
        .arg("filter")
        .arg("003@.0?")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false
[filter]
skip-invalid = true
"#,
        )
        .arg("filter")
        .arg("003@.0?")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false
[filter]
skip-invalid = false
"#,
        )
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0?")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_cardinality_op() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("#007N{ a == 'pnd' && v == 'zg'} == 2 && 003@.0?")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    for filter_expr in ["#047C <= 2", "#047C == 2", "#047C >= 2"] {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("filter")
            .arg(filter_expr)
            .arg("tests/data/121169502.dat")
            .assert();

        let expected = predicate::path::eq_file(Path::new(
            "tests/data/121169502.dat",
        ));
        assert.success().stdout(expected);
    }

    for filter_expr in
        ["#047C < 2", "#047C == 1", "#047C != 2", "#048C > 2"]
    {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("filter")
            .arg(filter_expr)
            .arg("tests/data/121169502.dat")
            .assert();

        assert
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
    }

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("#047C > 1")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("#047C < 4")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("008A{ #a == 2 }")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("008A{ #a < 2 }")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("008A{ #a > 2 }")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("008[AB]{ #a == 2 && a == 'f'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_filter_strsim() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("028A.d =* 'Heike'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("028A.d =* 'Heiko'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--strsim-threshold")
        .arg("99")
        .arg("028A.d =* 'Heiko'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--strsim-threshold")
        .arg("110")
        .arg("028A.d =* 'Heiko'")
        .arg("tests/data/121169502.dat")
        .assert();

    // error code 2 is set by clap-rs
    assert.failure().code(2).stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--strsim-threshold")
        .arg("abc")
        .arg("028A.d =* 'Heiko'")
        .arg("tests/data/121169502.dat")
        .assert();

    // error code 2 is set by clap-rs
    assert.failure().code(2).stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_allow_deny_listing_csv() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--allow-list")
        .arg("tests/data/allow_list.csv")
        .arg("003@.0 not in ['000008672', '119232022']")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--allow-list")
        .arg("tests/data/allow_list.csv")
        .arg("--deny-list")
        .arg("tests/data/deny_list.csv")
        .arg("003@.0?")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    // short options
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("-A")
        .arg("tests/data/allow_list.csv")
        .arg("-D")
        .arg("tests/data/deny_list.csv")
        .arg("003@.0?")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_filter_allow_deny_listing_arrow() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--allow-list")
        .arg("tests/data/allow_list.arrow")
        .arg("003@.0 not in ['000008672', '119232022']")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--allow-list")
        .arg("tests/data/allow_list.arrow")
        .arg("--deny-list")
        .arg("tests/data/deny_list.arrow")
        .arg("003@.0?")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    // short options
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("-A")
        .arg("tests/data/allow_list.arrow")
        .arg("-D")
        .arg("tests/data/deny_list.arrow")
        .arg("003@.0?")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected = predicate::path::eq_file(Path::new(
        "tests/data/1004916019.dat",
    ));
    assert.success().stdout(expected);

    Ok(())
}
