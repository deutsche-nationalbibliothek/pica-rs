use crate::support::{CommandBuilder, MatchResult};
use std::fs::read_to_string;
use tempfile::Builder;

#[test]
fn json_single_record() -> MatchResult {
    let expected = read_to_string("tests/data/1004916019.json").unwrap();

    CommandBuilder::new("json")
        .arg("tests/data/1004916019.dat")
        .with_stdout(expected.trim_end())
        .run()?;

    Ok(())
}

#[test]
fn json_write_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-json").tempdir().unwrap();
    let filename = tempdir.path().join("sample.json");

    CommandBuilder::new("json")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let expected = read_to_string("tests/data/1004916019.json").unwrap();
    assert_eq!(expected.trim_end(), read_to_string(filename).unwrap());

    Ok(())
}

#[test]
fn json_skip_invalid() -> MatchResult {
    CommandBuilder::new("json")
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .with_stdout("[]")
        .run()?;

    CommandBuilder::new("json")
        .with_config(
            r#"
[global]
skip-invalid = true
"#,
        )
        .arg("tests/data/invalid.dat")
        .with_stdout("[]")
        .run()?;

    CommandBuilder::new("json")
        .with_config(
            r#"
[json]
skip-invalid = true
"#,
        )
        .arg("tests/data/invalid.dat")
        .with_stdout("[]")
        .run()?;

    CommandBuilder::new("json")
        .with_config(
            r#"
[global]
skip-invalid = false

[json]
skip-invalid = true
"#,
        )
        .arg("tests/data/invalid.dat")
        .with_stdout("[]")
        .run()?;

    CommandBuilder::new("json")
        .with_config(
            r#"
[global]
skip-invalid = false

[json]
skip-invalid = false
"#,
        )
        .arg("--skip-invalid")
        .arg("tests/data/invalid.dat")
        .with_stdout("[]")
        .run()?;

    Ok(())
}

#[test]
fn json_invalid_file() -> MatchResult {
    CommandBuilder::new("json")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
