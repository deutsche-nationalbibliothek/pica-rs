use crate::support::{
    CommandBuilder, MatchResult, SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4, SAMPLE5,
    SAMPLE6, SAMPLE7,
};
use std::fs::{read_to_string, remove_file};
use tempfile::Builder;

#[test]
fn partition_by_bbg() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    assert_eq!(read_to_string("Ts1.dat").unwrap(), SAMPLE1);
    remove_file("Ts1.dat").unwrap();

    let mut exprected = String::new();
    exprected.push_str(SAMPLE2);
    exprected.push_str(SAMPLE7);

    assert_eq!(read_to_string("Tp1.dat").unwrap(), exprected);
    remove_file("Tp1.dat").unwrap();

    let mut exprected = String::new();
    exprected.push_str(SAMPLE3);
    exprected.push_str(SAMPLE4);
    exprected.push_str(SAMPLE5);
    exprected.push_str(SAMPLE6);

    assert_eq!(read_to_string("Tb1.dat").unwrap(), exprected);
    remove_file("Tb1.dat").unwrap();

    Ok(())
}

#[test]
fn partition_output_dir1() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path().join("bbg");

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    assert_eq!(read_to_string(outdir.join("Ts1.dat")).unwrap(), SAMPLE1);

    let mut exprected = String::new();
    exprected.push_str(SAMPLE2);
    exprected.push_str(SAMPLE7);

    assert_eq!(read_to_string(outdir.join("Tp1.dat")).unwrap(), exprected);

    let mut exprected = String::new();
    exprected.push_str(SAMPLE3);
    exprected.push_str(SAMPLE4);
    exprected.push_str(SAMPLE5);
    exprected.push_str(SAMPLE6);

    assert_eq!(read_to_string(outdir.join("Tb1.dat")).unwrap(), exprected);

    Ok(())
}

#[test]
fn partition_output_dir2() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path();

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    assert_eq!(read_to_string(outdir.join("Ts1.dat")).unwrap(), SAMPLE1);

    let mut exprected = String::new();
    exprected.push_str(SAMPLE2);
    exprected.push_str(SAMPLE7);

    assert_eq!(read_to_string(outdir.join("Tp1.dat")).unwrap(), exprected);

    let mut exprected = String::new();
    exprected.push_str(SAMPLE3);
    exprected.push_str(SAMPLE4);
    exprected.push_str(SAMPLE5);
    exprected.push_str(SAMPLE6);

    assert_eq!(read_to_string(outdir.join("Tb1.dat")).unwrap(), exprected);

    Ok(())
}

#[test]
fn partition_no_path() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("tests/data/dump.dat.gz")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn partition_invalid_path1() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("00!@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stderr("Pica Error: Invalid path expression\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn partition_invalid_path2() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args("--outdir /root/foo")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn partition_invalid_file() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
