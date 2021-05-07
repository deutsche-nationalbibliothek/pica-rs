use crate::support::{CommandBuilder, MatchResult, SAMPLE1, SAMPLE2};
use std::fs::{read_to_string, remove_file};
use tempfile::Builder;

#[test]
fn split_default() -> MatchResult {
    CommandBuilder::new("split")
        .arg("-s")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    assert_eq!(read_to_string("0.dat").unwrap(), SAMPLE1);
    remove_file("0.dat").unwrap();

    assert_eq!(read_to_string("1.dat").unwrap(), SAMPLE2);
    remove_file("1.dat").unwrap();

    Ok(())
}

#[test]
fn split_outdir() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-split").tempdir().unwrap();
    let outdir = tempdir.path();

    CommandBuilder::new("split")
        .arg("--skip-invalid")
        .arg("1")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    assert_eq!(read_to_string(outdir.join("0.dat")).unwrap(), SAMPLE1);
    remove_file(outdir.join("0.dat")).unwrap();

    assert_eq!(read_to_string(outdir.join("1.dat")).unwrap(), SAMPLE2);
    remove_file(outdir.join("1.dat")).unwrap();

    Ok(())
}

#[test]
fn split_template() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-split").tempdir().unwrap();
    let outdir = tempdir.path();

    CommandBuilder::new("split")
        .arg("--skip-invalid")
        .arg("1")
        .args("--template CHUNK_{}.dat")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    assert_eq!(read_to_string(outdir.join("CHUNK_0.dat")).unwrap(), SAMPLE1);
    remove_file(outdir.join("CHUNK_0.dat")).unwrap();

    assert_eq!(read_to_string(outdir.join("CHUNK_1.dat")).unwrap(), SAMPLE2);
    remove_file(outdir.join("CHUNK_1.dat")).unwrap();

    Ok(())
}

#[test]
fn split_invalid_chunk_size() -> MatchResult {
    CommandBuilder::new("split")
        .arg("abc")
        .arg("tests/data/dump.dat.gz")
        .with_status(101)
        .run()?;

    CommandBuilder::new("split")
        .arg("0")
        .arg("tests/data/dump.dat.gz")
        .with_stderr("error: chunk size < 1\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn split_invalid_file() -> MatchResult {
    CommandBuilder::new("split")
        .arg("100")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
