use crate::support::{CommandBuilder, MatchResult};
use std::fs::read_to_string;
use tempfile::Builder;

#[test]
fn select_one_column() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("1004916019\n")
        .with_stdout("119232022\n")
        .with_stdout("000008672\n")
        .with_stdout("000016586\n")
        .with_stdout("000016756\n")
        .with_stdout("000009229\n")
        .with_stdout("121169502\n")
        .run()?;

    Ok(())
}

#[test]
fn select_two_columns() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("1004916019,Ts1\n")
        .with_stdout("119232022,Tp1\n")
        .with_stdout("000008672,Tb1\n")
        .with_stdout("000016586,Tb1\n")
        .with_stdout("000016756,Tb1\n")
        .with_stdout("000009229,Tb1\n")
        .with_stdout("121169502,Tp1\n")
        .run()?;

    Ok(())
}

#[test]
fn select_static_selector() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0, 'foo', 002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("1004916019,foo,Ts1\n")
        .with_stdout("119232022,foo,Tp1\n")
        .with_stdout("000008672,foo,Tb1\n")
        .with_stdout("000016586,foo,Tb1\n")
        .with_stdout("000016756,foo,Tb1\n")
        .with_stdout("000009229,foo,Tb1\n")
        .with_stdout("121169502,foo,Tp1\n")
        .run()?;

    Ok(())
}

#[test]
fn select_repeated_field() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0, 065R.9")
        .arg("tests/data/119232022.dat.gz")
        .with_stdout("119232022,040743357\n119232022,040743357\n")
        .run()?;

    Ok(())
}

#[test]
fn select_repeated_subfield() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0, 008A.a")
        .arg("tests/data/119232022.dat.gz")
        .with_stdout("119232022,s\n119232022,z\n119232022,f\n")
        .run()?;

    Ok(())
}

#[test]
fn select_empty_row() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("012A.a, 013A.a")
        .arg("tests/data/119232022.dat.gz")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn select_filter() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0, 065R{4 == 'ortg' && 7 == 'Tgz', 9}")
        .arg("tests/data/119232022.dat.gz")
        .with_stdout("119232022,040743357\n")
        .run()?;

    Ok(())
}

#[test]
fn select_occurrence_matcher() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("047A/*.e")
        .arg("tests/data/119232022.dat")
        .with_stdout("DE-386\n")
        .run()?;

    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("047A/03.e")
        .arg("tests/data/119232022.dat")
        .with_stdout("DE-386\n")
        .run()?;

    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("047A/01-03.e")
        .arg("tests/data/119232022.dat")
        .with_stdout("DE-386\n")
        .run()?;

    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("047A/01-04.e")
        .arg("tests/data/119232022.dat")
        .with_stdout("DE-386\n")
        .run()?;

    Ok(())
}

#[test]
fn select_header() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .args("--header idn,bbg")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("idn,bbg\n")
        .with_stdout("1004916019,Ts1\n")
        .with_stdout("119232022,Tp1\n")
        .with_stdout("000008672,Tb1\n")
        .with_stdout("000016586,Tb1\n")
        .with_stdout("000016756,Tb1\n")
        .with_stdout("000009229,Tb1\n")
        .with_stdout("121169502,Tp1\n")
        .run()?;

    Ok(())
}

#[test]
fn select_tab_separated() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("--tsv")
        .args("--header idn,bbg")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("idn\tbbg\n")
        .with_stdout("1004916019\tTs1\n")
        .with_stdout("119232022\tTp1\n")
        .with_stdout("000008672\tTb1\n")
        .with_stdout("000016586\tTb1\n")
        .with_stdout("000016756\tTb1\n")
        .with_stdout("000009229\tTb1\n")
        .with_stdout("121169502\tTp1\n")
        .run()?;

    Ok(())
}

#[test]
fn select_write_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-select").tempdir().unwrap();
    let filename = tempdir.path().join("sample.csv");

    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    assert_eq!(
        read_to_string(filename).unwrap(),
        "1004916019,Ts1
119232022,Tp1
000008672,Tb1
000016586,Tb1
000016756,Tb1
000009229,Tb1
121169502,Tp1
"
    );

    Ok(())
}

#[test]
fn select_skip_invalid() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .with_stdout_empty()
        .run()?;

    CommandBuilder::new("select")
        .with_config(
            r#"
[global]
skip-invalid = true
"#,
        )
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .with_stdout_empty()
        .run()?;

    CommandBuilder::new("select")
        .with_config(
            r#"
[select]
skip-invalid = true
"#,
        )
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .with_stdout_empty()
        .run()?;

    CommandBuilder::new("select")
        .with_config(
            r#"
[global]
skip-invalid = false

[select]
skip-invalid = true
"#,
        )
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .with_stdout_empty()
        .run()?;

    CommandBuilder::new("select")
        .with_config(
            r#"
[global]
skip-invalid = false

[select]
skip-invalid = false
"#,
        )
        .arg("--skip-invalid")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn select_invalid_file() -> MatchResult {
    CommandBuilder::new("select")
        .arg("003@.0")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
