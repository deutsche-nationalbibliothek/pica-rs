use crate::support::{CommandBuilder, MatchResult};
use std::fs::read_to_string;
use tempfile::Builder;

#[test]
fn frequency_default() -> MatchResult {
    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("Tb1,4\n")
        .with_stdout("Tp1,2\n")
        .with_stdout("Ts1,1\n")
        .run()?;

    Ok(())
}

#[test]
fn frequency_reverse() -> MatchResult {
    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .arg("--reverse")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("Ts1,1\n")
        .with_stdout("Tp1,2\n")
        .with_stdout("Tb1,4\n")
        .run()?;

    Ok(())
}

#[test]
fn frequency_limit() -> MatchResult {
    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .args("--limit 2")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("Tb1,4\n")
        .with_stdout("Tp1,2\n")
        .run()?;

    Ok(())
}

#[test]
fn frequency_threshold() -> MatchResult {
    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .args("--threshold 1")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("Tb1,4\n")
        .with_stdout("Tp1,2\n")
        .run()?;

    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .args("--threshold 2")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("Tb1,4\n")
        .run()?;

    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .args("--threshold 3")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout("Tb1,4\n")
        .run()?;

    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .args("--threshold 999")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn frequency_invalid_threshold() -> MatchResult {
    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .args("--threshold abc")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stderr(
            "error: Invalid threshold value, expected unsigned integer.\n",
        )
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn frequency_no_values() -> MatchResult {
    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .arg("012A.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn frequency_write_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-frequency").tempdir().unwrap();
    let filename = tempdir.path().join("sample.csv");

    CommandBuilder::new("frequency")
        .arg("--skip-invalid")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    assert_eq!(read_to_string(filename).unwrap(), "Tb1,4\nTp1,2\nTs1,1\n");

    Ok(())
}

#[test]
fn frequency_invalid_file() -> MatchResult {
    CommandBuilder::new("frequency")
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
