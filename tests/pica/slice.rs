use crate::support::{
    CommandBuilder, MatchResult, SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4, SAMPLE5,
    SAMPLE6, SAMPLE7,
};
use std::fs::read_to_string;
use tempfile::Builder;

#[test]
fn slice_default() -> MatchResult {
    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .with_stdout(SAMPLE3)
        .with_stdout(SAMPLE4)
        .with_stdout(SAMPLE5)
        .with_stdout(SAMPLE6)
        .with_stdout(SAMPLE7)
        .run()?;

    Ok(())
}

#[test]
fn slice_write_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-slice").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--start 1")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    let mut expected = String::new();
    expected.push_str(SAMPLE2);
    expected.push_str(SAMPLE3);
    expected.push_str(SAMPLE4);
    expected.push_str(SAMPLE5);
    expected.push_str(SAMPLE6);
    expected.push_str(SAMPLE7);

    assert_eq!(read_to_string(filename).unwrap(), expected);
    Ok(())
}

#[test]
fn slice_start_option() -> MatchResult {
    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--start 0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .with_stdout(SAMPLE3)
        .with_stdout(SAMPLE4)
        .with_stdout(SAMPLE5)
        .with_stdout(SAMPLE6)
        .with_stdout(SAMPLE7)
        .run()?;

    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--start 1")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE2)
        .with_stdout(SAMPLE3)
        .with_stdout(SAMPLE4)
        .with_stdout(SAMPLE5)
        .with_stdout(SAMPLE6)
        .with_stdout(SAMPLE7)
        .run()?;

    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--start 999")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn slice_invalid_start_option() -> MatchResult {
    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--start abc")
        .arg("tests/data/dump.dat.gz")
        .with_stderr("error: invalid start option\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn slice_end_option() -> MatchResult {
    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--end 1")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .run()?;

    // invalid record on position 1
    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--end 2")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .run()?;

    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--end 3")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--end 999")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .with_stdout(SAMPLE3)
        .with_stdout(SAMPLE4)
        .with_stdout(SAMPLE5)
        .with_stdout(SAMPLE6)
        .with_stdout(SAMPLE7)
        .run()?;

    Ok(())
}

#[test]
fn slice_invalid_end_option() -> MatchResult {
    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--end abc")
        .arg("tests/data/dump.dat.gz")
        .with_stderr("error: invalid end option\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn slice_length_option() -> MatchResult {
    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--length 1")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .run()?;

    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--length 2")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--length 100")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .with_stdout(SAMPLE3)
        .with_stdout(SAMPLE4)
        .with_stdout(SAMPLE5)
        .with_stdout(SAMPLE6)
        .with_stdout(SAMPLE7)
        .run()?;

    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--start 1")
        .args("--length 1")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn slice_invalid_length_option() -> MatchResult {
    CommandBuilder::new("slice")
        .arg("--skip-invalid")
        .args("--length abc")
        .arg("tests/data/dump.dat.gz")
        .with_stderr("error: invalid length option\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn slice_invalid_file() -> MatchResult {
    CommandBuilder::new("slice")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
