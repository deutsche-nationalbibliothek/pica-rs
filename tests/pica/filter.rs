use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_filter_equal_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '121169502'")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@{0 == '121169502'}")
        .arg("tests/data/dump.dat.gz")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '123456789X'")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_strict_equal_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 === '121169502'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@{0 === '121169502'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("008A.a === 's'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_not_equal_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 != '12116950X'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@{0 != '12116950X'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 != '121169502'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("008B.a != 'x'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("008B.b != 'x'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_filter_regex_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("001A.0 =~ '^\\\\d{4}:\\\\d{2}-\\\\d{2}-\\\\d{2}$'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("001A{0 =~ '^\\\\d{4}:\\\\d{2}-\\\\d{2}-\\\\d{2}$'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("001A.0 =~ '^\\\\d{5}:\\\\d{2}-\\\\d{2}-\\\\d{2}$'")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("001A.0 =~ '\\d{a}'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::eq("error: invalid filter: \"001A.0 =~ \'\\d{a}\'\"\n");

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(expected);

    Ok(())
}

#[test]
fn pica_filter_starts_with_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@.0 =^ 'Tp'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@{0 =^ 'Tp'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@.0 =^ 'Tb'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_ends_with_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("001A.0 =$ '-99'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("001A{0 =$ '-99'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@.0 =$ '-10'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_in_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@.0 in ['Tp3', 'Tp2', 'Tp1']")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@{0 in ['Tp3', 'Tp2', 'Tp1']}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@.0 in ['Tp2', 'Tp3']")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_not_in_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@.0 not in ['Tp3', 'Tp2']")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@{0 not in ['Tp3', 'Tp2']}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("002@.0 not in ['Tp2', 'Tp1', 'Tp3']")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_exists_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("047A/03.e?")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("047A/03{e?}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("047A/03.f?")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_not_operator() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("!047A/03.f?")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("047A/03{!f?}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("!047A/03.e?")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_and_connective() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '121169502' && 002@.0 == 'Tp1'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '12116950X' && 002@.0 == 'Tp1'")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '121169502' && 002@.0 == 'Tp2'")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '12116950X' && 002@.0 == 'Tp2'")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("007N{a == 'gnd' && 0 == '183361946'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("007N{a == 'swd' && 0 == '183361946'}")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("007N{a == 'gnd' && 0 == '18336194X'}")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("007N{a == 'swd' && 0 == '18336194X'}")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_or_connective() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '121169502' || 002@.0 == 'Tp1'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '121169502' || 002@.0 == 'Tp2'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '12116950X' || 002@.0 == 'Tp1'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '12116950X' || 002@.0 == 'Tp2'")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("007N{a == 'gnd' || 0 == '183361946'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("007N{a == 'swd' || 0 == '183361946'}")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("007N{a == 'gnd' || 0 == '18336194X'}")
        .arg("tests/data/121169502.dat")
        .assert();
    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("007N{a == 'xxx' || 0 == '18336194X'}")
        .arg("tests/data/121169502.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_filter_groups() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("((003@.0 == '121169502'))")
        .arg("tests/data/121169502.dat")
        .assert();
    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("(003@.0 == '121169502' && (002@.0 == 'Tp2' || 002@.0 == 'Tp1'))")
        .arg("tests/data/121169502.dat")
        .assert();
    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    let mut cmd = Command::cargo_bin("pica")?;
    let filter = r#"003@.0 == '121169502' && 
         007N{a? && (0 == '121169502' || 0 == '183361946')}"#;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg(filter)
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_filter_invert_match() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--invert-match")
        .arg("003@.0 == '121169502'")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--skip-invalid")
        .arg("--invert-match")
        .arg("003@.0 != '121169502'")
        .arg("tests/data/121169502.dat")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/121169502.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_filter_read_gzip() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/1004916019.dat.gz")
        .assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
    assert.success().stdout(expected);

    Ok(())
}

#[test]
fn pica_filter_write_gzip() -> TestResult {
    let filename = Builder::new().suffix(".gz").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("filter")
        .arg("--output")
        .arg(filename_str)
        .arg("003@.0?")
        .arg("tests/data/1004916019.dat")
        .assert();
    assert.success();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("cat").arg(filename_str).assert();

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
    assert.success().stdout(expected);

    Ok(())
}

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

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
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

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
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

    let expected =
        predicate::path::eq_file(Path::new("tests/data/1004916019.dat"));
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

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::eq(
            "error: Invalid limit value, expected unsigned integer.\n",
        ));

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
        .stderr(predicate::eq("Pica Error: Invalid record on line 2.\n"));

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
