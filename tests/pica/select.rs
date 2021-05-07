use crate::support::{CommandBuilder, MatchResult};
use std::fs::read_to_string;
use tempfile::Builder;

#[test]
fn select_one_column() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("1004916019\n119232022\n")
        .run()?;

    Ok(())
}

#[test]
fn select_two_columns() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("1004916019,Ts1\n119232022,Tp1\n")
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
fn select_header() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .args("--header idn,bbg")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("idn,bbg\n1004916019,Ts1\n119232022,Tp1\n")
        .run()?;

    Ok(())
}

#[test]
fn select_tab_separated() -> MatchResult {
    CommandBuilder::new("select")
        .arg("--skip-invalid")
        .arg("--tsv")
        .arg("003@.0, 002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("1004916019\tTs1\n119232022\tTp1\n")
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
        "1004916019,Ts1\n119232022,Tp1\n"
    );
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
