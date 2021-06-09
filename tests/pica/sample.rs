use crate::support::{
    CommandBuilder, MatchResult, SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4, SAMPLE5,
    SAMPLE6, SAMPLE7,
};
use flate2::read::GzDecoder;
use std::fs::{read_to_string, File};
use std::io::Read;
use tempfile::Builder;

#[test]
fn sample_single_record() -> MatchResult {
    CommandBuilder::new("sample")
        .arg("1")
        .arg("tests/data/1004916019.dat")
        .with_stdout(SAMPLE1)
        .run()?;

    Ok(())
}

#[test]
fn sample_multiple_records() -> MatchResult {
    CommandBuilder::new("sample")
        .arg("--skip-invalid")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_one_of(
            [
                SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4, SAMPLE5, SAMPLE6, SAMPLE7,
            ]
            .to_vec(),
        )
        .run()?;

    Ok(())
}

#[test]
fn sample_size_le_len() -> MatchResult {
    CommandBuilder::new("sample")
        .arg("--skip-invalid")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_one_of(
            [
                SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4, SAMPLE5, SAMPLE6, SAMPLE7,
            ]
            .to_vec(),
        )
        .run()?;

    Ok(())
}

#[test]
fn sample_size_eq_len() -> MatchResult {
    CommandBuilder::new("sample")
        .arg("--skip-invalid")
        .arg("2")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_lines(2)
        .run()?;

    Ok(())
}

#[test]
fn sample_size_gt_len() -> MatchResult {
    CommandBuilder::new("sample")
        .arg("--skip-invalid")
        .arg("8")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_lines(7)
        .run()?;

    Ok(())
}

#[test]
fn sample_write_plain_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-sample").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("sample")
        .arg("--skip-invalid")
        .arg("1")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    assert_eq!(
        read_to_string("tests/data/1004916019.dat").unwrap(),
        read_to_string(filename).unwrap()
    );

    Ok(())
}

#[test]
fn sample_write_gzip_output() -> MatchResult {
    // file extension
    let tempdir = Builder::new().prefix("pica-sample").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat.gz");

    CommandBuilder::new("sample")
        .arg("--skip-invalid")
        .arg("1")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let mut gz = GzDecoder::new(File::open(filename).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // gzip flag
    let tempdir = Builder::new().prefix("pica-sample").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("sample")
        .arg("--skip-invalid")
        .arg("--gzip")
        .arg("1")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let mut gz = GzDecoder::new(File::open(filename).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    Ok(())
}

#[test]
fn sample_invalid_sample_size() -> MatchResult {
    CommandBuilder::new("sample")
        .arg("0")
        .arg("tests/data/dump.dat.gz")
        .with_status(1)
        .run()?;

    CommandBuilder::new("sample")
        .arg("\\-1")
        .arg("tests/data/dump.dat.gz")
        .with_status(1)
        .run()?;

    CommandBuilder::new("sample")
        .arg("abc")
        .arg("tests/data/dump.dat.gz")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn sample_invalid_file() -> MatchResult {
    CommandBuilder::new("sample")
        .arg("100")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
