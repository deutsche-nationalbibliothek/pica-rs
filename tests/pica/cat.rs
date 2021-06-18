use crate::support::{CommandBuilder, MatchResult, SAMPLE1, SAMPLE2};
use flate2::read::GzDecoder;
use std::fs::{read_to_string, File};
use std::io::Read;
use tempfile::Builder;

#[test]
fn cat_no_file() -> MatchResult {
    CommandBuilder::new("cat").with_status(2).run()?;
    Ok(())
}

#[test]
fn cat_single_file() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/1004916019.dat")
        .with_stdout(SAMPLE1)
        .run()?;

    Ok(())
}

#[test]
fn cat_multiple_files() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn cat_gzip_file() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/119232022.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn cat_missing_file() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/123456789X.dat")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn cat_invalid_file() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn cat_skip_invalid() -> MatchResult {
    CommandBuilder::new("cat")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/invalid.dat")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("cat")
        .with_config(
            r#"[cat]
skip-invalid = true
"#,
        )
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/invalid.dat")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("cat")
        .with_config(
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/invalid.dat")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("cat")
        .with_config(
            r#"[global]
skip-invalid = false

[cat]
skip-invalid = true
"#,
        )
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/invalid.dat")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("cat")
        .with_config(
            r#"[global]
skip-invalid = false

[cat]
skip-invalid = false
"#,
        )
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/invalid.dat")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn cat_write_plain_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-cat-plain").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("cat")
        .arg("--skip-invalid")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/119232022.dat")
        .with_stdout_empty()
        .run()?;

    assert_eq!(read_to_string(filename).unwrap(), SAMPLE2);
    Ok(())
}

#[test]
fn cat_write_gzip_output() -> MatchResult {
    // file extension
    let tempdir = Builder::new().prefix("pica-cat-gzip").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat.gz");

    CommandBuilder::new("cat")
        .arg("--skip-invalid")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let mut gz = GzDecoder::new(File::open(filename).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // gzip-flag
    let tempdir = Builder::new().prefix("pica-cat-gzip").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("cat")
        .arg("--skip-invalid")
        .arg("--gzip")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let mut gz = GzDecoder::new(File::open(filename).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // config
    let tempdir = Builder::new().prefix("pica-cat-gzip").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("cat")
        .arg("--skip-invalid")
        .with_config(
            r#"[cat]
gzip = true
"#,
        )
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let mut gz = GzDecoder::new(File::open(filename).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // cli flag overwrites config
    let tempdir = Builder::new().prefix("pica-cat-gzip").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("cat")
        .arg("--skip-invalid")
        .with_config(
            r#"[cat]
gzip = false
"#,
        )
        .arg("--gzip")
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
