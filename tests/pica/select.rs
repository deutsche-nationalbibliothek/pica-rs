use std::fs::read_to_string;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_select_one_column() -> TestResult {
    for selector in ["003@.0", "003@$0", "003@ $0"] {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("select")
            .arg("--skip-invalid")
            .arg(selector)
            .arg("tests/data/dump.dat.gz")
            .assert();

        assert.success().stderr(predicate::str::is_empty()).stdout(
            r#"1004916019
119232022
000008672
000016586
000016756
000009229
121169502
"#,
        );
    }

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--skip-invalid")
        .arg("003@0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stderr("Don\'t use lazy syntax!\n").stdout(
        r#"1004916019
119232022
000008672
000016586
000016756
000009229
121169502
"#,
    );

    Ok(())
}

#[test]
fn pica_select_two_columns() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--skip-invalid")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(
        r#"1004916019,Ts1
119232022,Tp1
000008672,Tb1
000016586,Tb1
000016756,Tb1
000009229,Tb1
121169502,Tp1
"#,
    );

    Ok(())
}

#[test]
fn pica_select_static_selector() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0, 065R.9")
        .arg("tests/data/119232022.dat.gz")
        .assert();

    assert.success().stdout(
        r#"119232022,040743357
119232022,040743357
"#,
    );

    Ok(())
}

#[test]
fn pica_select_repeated_field() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0, 065R.9")
        .arg("tests/data/119232022.dat.gz")
        .assert();

    assert.success().stdout(
        r#"119232022,040743357
119232022,040743357
"#,
    );

    Ok(())
}

#[test]
fn pica_select_repeated_subfield() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0, 008A.a")
        .arg("tests/data/119232022.dat.gz")
        .assert();

    assert.success().stdout(
        r#"119232022,s
119232022,z
119232022,f
"#,
    );

    Ok(())
}

#[test]
fn pica_select_empty_row() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("012A.a, 013A.a")
        .arg("tests/data/119232022.dat.gz")
        .assert();

    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_select_filter() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0, 065R{4 == 'ortg' && 7 == 'Tgz', 9}")
        .arg("tests/data/119232022.dat.gz")
        .assert();

    assert.success().stdout("119232022,040743357\n");

    Ok(())
}

#[test]
fn pica_select_occurrence_matcher() -> TestResult {
    // any
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("047A/*.e")
        .arg("tests/data/119232022.dat.gz")
        .assert();
    assert.success().stdout("DE-386\n");

    // sepecial case "/00"
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("001U/00.0")
        .arg("tests/data/119232022.dat.gz")
        .assert();
    assert.success().stdout("utf8\n");

    // explicit
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("047A/03.e")
        .arg("tests/data/119232022.dat.gz")
        .assert();
    assert.success().stdout("DE-386\n");

    // range
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("047A/01-03.e")
        .arg("tests/data/119232022.dat.gz")
        .assert();
    assert.success().stdout("DE-386\n");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("047A/01-04.e")
        .arg("tests/data/119232022.dat.gz")
        .assert();
    assert.success().stdout("DE-386\n");

    Ok(())
}

#[test]
fn pica_select_tag_pattern() -> TestResult {
    // any
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("001[AB].0")
        .arg("tests/data/119232022.dat.gz")
        .assert();
    assert.success().stdout(
        r#"0386:16-03-95
8999:20-07-20
"#,
    );

    Ok(())
}

#[test]
fn pica_select_header() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--skip-invalid")
        .arg("--header")
        .arg("idn,bbg")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(
        r#"idn,bbg
1004916019,Ts1
119232022,Tp1
000008672,Tb1
000016586,Tb1
000016756,Tb1
000009229,Tb1
121169502,Tp1
"#,
    );

    Ok(())
}

#[test]
fn pica_select_tab_separated() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--skip-invalid")
        .arg("--header")
        .arg("idn,bbg")
        .arg("--tsv")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stdout(
        "idn\tbbg
1004916019\tTs1
119232022\tTp1
000008672\tTb1
000016586\tTb1
000016756\tTb1
000009229\tTb1
121169502\tTp1
",
    );

    Ok(())
}

#[test]
fn pica_select_no_empty_columns() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--no-empty-columns")
        .arg("003@.0, 001[AB]{0, t}")
        .arg("tests/data/119232022.dat.gz")
        .assert();

    assert
        .success()
        .stdout("119232022,8999:20-07-20,13:19:49.000\n");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0, 001[AB]{0, t}")
        .arg("tests/data/119232022.dat.gz")
        .assert();

    assert.success().stdout(
        r#"119232022,0386:16-03-95,
119232022,8999:20-07-20,13:19:49.000
"#,
    );

    Ok(())
}

#[test]
fn pica_select_ignore_case() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0, 050E{a == 'internet', a}")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout("121169502,\n");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--ignore-case")
        .arg("003@.0, 050E{a == 'internet', a}")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout("121169502,Internet\n");

    Ok(())
}

#[test]
fn pica_select_unique() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0,028[A@].d")
        .arg("tests/data/121169502.dat")
        .assert();

    assert
        .success()
        .stdout("121169502,Heike\n121169502,Heike\n");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--unique")
        .arg("003@.0,028[A@].d")
        .arg("tests/data/121169502.dat")
        .assert();

    assert.success().stdout("121169502,Heike\n");

    Ok(())
}

#[test]
fn pica_select_multiple_files() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0, 002@.0")
        .arg("tests/data/119232022.dat.gz")
        .arg("tests/data/1004916019.dat.gz")
        .assert();

    assert.success().stdout(
        r#"119232022,Tp1
1004916019,Ts1
"#,
    );

    Ok(())
}

#[test]
fn pica_select_write_output() -> TestResult {
    let filename = Builder::new().suffix(".csv").tempfile()?;
    let filename_str = filename.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--skip-invalid")
        .arg("--output")
        .arg(filename_str)
        .arg("003@.0, 'foo', 002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let expected = read_to_string("tests/data/dump.csv")?;
    let expected = if cfg!(windows) {
        expected.replace('\r', "")
    } else {
        expected
    };

    let actual = read_to_string(filename_str)?;
    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_select_translit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("029A.a")
        .arg("tests/data/004732650.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout("Goethe-Universita\u{308}t Frankfurt am Main\n");

    let expected = vec![
        ("nfd", "Goethe-Universita\u{308}t Frankfurt am Main\n"),
        ("nfkd", "Goethe-Universita\u{308}t Frankfurt am Main\n"),
        ("nfc", "Goethe-Universität Frankfurt am Main\n"),
        ("nfkc", "Goethe-Universität Frankfurt am Main\n"),
    ];

    for (translit, output) in expected {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("select")
            .arg("--translit")
            .arg(translit)
            .arg("029A.a")
            .arg("tests/data/004732650.dat.gz")
            .assert();

        assert
            .success()
            .stderr(predicate::str::is_empty())
            .stdout(output);
    }

    let expected = vec![
        ("nfd", "Goethe-Universita\u{308}t Frankfurt am Main\n"),
        ("nfkd", "Goethe-Universita\u{308}t Frankfurt am Main\n"),
        ("nfc", "Goethe-Universität Frankfurt am Main\n"),
        ("nfkc", "Goethe-Universität Frankfurt am Main\n"),
    ];

    for (translit, output) in expected {
        let mut cmd = Command::cargo_bin("pica")?;
        let assert = cmd
            .arg("select")
            .arg("--translit")
            .arg(translit)
            .arg("029A.a")
            .arg("tests/data/004732650-nfc.dat.gz")
            .assert();

        assert
            .success()
            .stderr(predicate::str::is_empty())
            .stdout(output);
    }

    Ok(())
}
#[test]
fn pica_select_where() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--skip-invalid")
        .arg("003@.0")
        .arg("--where")
        .arg("003@.0 =^ '0'")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert.success().stderr(predicate::str::is_empty()).stdout(
        r#"000008672
000016586
000016756
000009229
"#,
    );

    Ok(())
}

#[test]
fn pica_select_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--skip-invalid")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .assert();
    assert
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr("Pica Error: Invalid record on line 1.\n");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("select")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[select]
skip-invalid = true
"#,
        )
        .arg("select")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[select]
skip-invalid = true
"#,
        )
        .arg("select")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[select]
skip-invalid = false
"#,
        )
        .arg("select")
        .arg("--skip-invalid")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .assert();
    assert.success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn pica_select_invalid_selector() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("select")
        .arg("--skip-invalid")
        .arg("003@.0, 'foo, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr("error: invalid select list: 003@.0, \'foo, 002@.0\n");

    Ok(())
}
