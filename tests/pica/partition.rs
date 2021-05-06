use crate::support::{CommandBuilder, MatchResult, SAMPLE1, SAMPLE2};
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

    assert_eq!(read_to_string("Tp1.dat").unwrap(), SAMPLE2);
    remove_file("Tp1.dat").unwrap();

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
    assert_eq!(read_to_string(outdir.join("Tp1.dat")).unwrap(), SAMPLE2);

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
    assert_eq!(read_to_string(outdir.join("Tp1.dat")).unwrap(), SAMPLE2);

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
fn partition_invalid_path() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("00!@.0")
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
