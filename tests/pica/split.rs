use crate::support::{
    CommandBuilder, MatchResult, SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4, SAMPLE5,
    SAMPLE6, SAMPLE7,
};
use std::fs::{read_to_string, remove_file};
use tempfile::Builder;

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

    let expected = [
        ("0.dat", SAMPLE1),
        ("1.dat", SAMPLE2),
        ("2.dat", SAMPLE3),
        ("3.dat", SAMPLE4),
        ("4.dat", SAMPLE5),
        ("5.dat", SAMPLE6),
        ("6.dat", SAMPLE7),
    ];

    for (filename, sample) in expected.iter() {
        assert_eq!(read_to_string(outdir.join(filename)).unwrap(), *sample);
        remove_file(outdir.join(filename)).unwrap();
    }

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

    let expected = [
        ("CHUNK_0.dat", SAMPLE1),
        ("CHUNK_1.dat", SAMPLE2),
        ("CHUNK_2.dat", SAMPLE3),
        ("CHUNK_3.dat", SAMPLE4),
        ("CHUNK_4.dat", SAMPLE5),
        ("CHUNK_5.dat", SAMPLE6),
        ("CHUNK_6.dat", SAMPLE7),
    ];

    for (filename, sample) in expected.iter() {
        assert_eq!(read_to_string(outdir.join(filename)).unwrap(), *sample);
        remove_file(outdir.join(filename)).unwrap();
    }

    Ok(())
}

#[test]
fn split_invalid_chunk_size() -> MatchResult {
    CommandBuilder::new("split")
        .arg("abc")
        .arg("tests/data/dump.dat.gz")
        .with_stderr("error: invalid chunk size\n")
        .with_status(1)
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
