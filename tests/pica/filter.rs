use crate::support::{CommandBuilder, MatchResult, SAMPLE1, SAMPLE2, SAMPLE7};
use flate2::read::GzDecoder;
use std::fs::{read_to_string, File};
use std::io::Read;
use tempfile::Builder;

#[test]
fn filter_equal() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .arg("  003@.0 == '1004916019' ")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .run()?;

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '1004916020'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_strict_equal() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("002@.0 === 'Tp1'")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("008A.a === 's'")
        .arg("tests/data/119232022.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_not_equal() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("003@.0 != '1004916019'")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    CommandBuilder::new("filter")
        .arg("003@.0 != '1004916019'")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn filter_regex() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("002@.0 =~ '^T[bp]1$'")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("002@.0 =~ '^T[bp]1$'")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_starts_with() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("002@.0 =^ 'Tp1'")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("002@.0 =^ 'Tp1'")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_ends_with() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("002@.0 =$ 'p1'")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("002@.0 =$ 'p1'")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_in() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("002@.0 in ['Tp1', 'Ts1']")
        .arg("tests/data/1004916019.dat")
        .with_stdout(SAMPLE1)
        .run()?;

    CommandBuilder::new("filter")
        .arg("002@.0 in ['Tp1', 'Ts1']")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("002@.0 in ['Tp2', 'Ts1']")
        .arg("tests/data/119232022.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_complex_comparision() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("065R{ 4 == 'ortg' && 9 == '040743357'}")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("065R{ 4 == 'ortg' && 9 == '040743357'}")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_and_connective() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("003@.0 == '119232022' && 002@.0 =^ 'Tp' && 001U.0 == 'utf8'")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("003@.0 == '119232022' && 065R{ 4 == 'ortg' && 9 == '040743357'}")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("003@.0 == '119232022' && 002@.0 =^ 'Ts'")
        .arg("tests/data/119232022.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_or_connective() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("003@.0 == '119232022' || 003@.0 == '1004916019' || 003@.0 == '123X'")
        .arg("tests/data/1004916019.dat")
        .with_stdout(SAMPLE1)
        .run()?;

    CommandBuilder::new("filter")
        .arg("003@{0 == '119232022' || 0 == '1004916019'}")
        .arg("tests/data/1004916019.dat")
        .with_stdout(SAMPLE1)
        .run()?;

    CommandBuilder::new("filter")
        .arg("003@.0 == '119232022' || 003@.0 == '1004916019'")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn filter_grouped() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("(003@.0 == '119232022' && 002@.0 == 'Tp1') || 003@.0 == '1004916019'")
        .arg("tests/data/1004916019.dat")
        .with_stdout(SAMPLE1)
        .run()?;

    CommandBuilder::new("filter")
        .arg("(003@.0 == '119232022' && 002@.0 == 'Tp1') || 003@.0 == '1004916019'")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("(((003@.0 == '119232022' && 002@.0 == 'Tp1')))")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn filter_exists() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("065R?")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("065R.9?")
        .arg("tests/data/119232022.dat")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("065R?")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    CommandBuilder::new("filter")
        .arg("065R.9?")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    Ok(())
}

#[test]
fn filter_not() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("!065R?")
        .arg("tests/data/119232022.dat")
        .with_stdout_empty()
        .run()?;

    CommandBuilder::new("filter")
        .arg("!065R.9?")
        .arg("tests/data/119232022.dat")
        .with_stdout_empty()
        .run()?;

    CommandBuilder::new("filter")
        .arg("!(065R?)")
        .arg("tests/data/1004916019.dat")
        .with_stdout(SAMPLE1)
        .run()?;

    Ok(())
}

#[test]
fn filter_invalid_filter() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == ''123456789X'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .with_stderr("error: invalid filter: \"003@.0 == ''123456789X'\"\n")
        .with_status(1)
        .run()?;

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .arg("002@.0 =~ '^O(?!lfo)")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .with_stderr("error: invalid filter: \"002@.0 =~ '^O(?!lfo)\"\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn filter_missing_file() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '119232022'")
        .arg("tests/data/123456789X.dat")
        .with_stdout_empty()
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn filter_invalid_file() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("003@.0 == '123456789X'")
        .arg("tests/data/invalid.dat")
        .with_stdout_empty()
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn filter_skip_invalid() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .arg("003@.0 == '119232022'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .with_config(
            r#"[filter]
skip-invalid = true
"#,
        )
        .arg("003@.0 == '119232022'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .with_config(
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("003@.0 == '119232022'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .with_config(
            r#"[global]
skip-invalid = false

[filter]
skip-invalid = true
"#,
        )
        .arg("003@.0 == '119232022'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .with_config(
            r#"[global]
skip-invalid = false

[filter]
skip-invalid = false
"#,
        )
        .arg("--skip-invalid")
        .arg("003@.0 == '119232022'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}

#[test]
fn filter_write_plain_output() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-filter").tempdir().unwrap();
    let filename = tempdir.path().join("sample2.dat");

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("003@.0 == '119232022'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout_empty()
        .run()?;

    assert_eq!(read_to_string(filename).unwrap(), SAMPLE2);
    Ok(())
}

#[test]
fn filter_write_gzip_output() -> MatchResult {
    // file extension
    let tempdir = Builder::new().prefix("pica-filter-gzip").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat.gz");

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let mut gz = GzDecoder::new(File::open(filename).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // gzip-flag
    let tempdir = Builder::new().prefix("pica-filter-gzip").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .arg("--gzip")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let mut gz = GzDecoder::new(File::open(filename).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // config
    let tempdir = Builder::new().prefix("pica-filter-gzip").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .with_config(
            r#"[filter]
gzip = true
"#,
        )
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("003@.0 == '1004916019'")
        .arg("tests/data/1004916019.dat")
        .with_stdout_empty()
        .run()?;

    let mut gz = GzDecoder::new(File::open(filename).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // cli flag overwrites config
    let tempdir = Builder::new().prefix("pica-filter-gzip").tempdir().unwrap();
    let filename = tempdir.path().join("sample.dat");

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .with_config(
            r#"[filter]
gzip = false
"#,
        )
        .arg("--gzip")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("003@.0 == '1004916019'")
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
fn filter_invert_match() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .arg("--invert-match")
        .arg("003@.0 =^ '0000'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .with_stdout(SAMPLE7)
        .run()?;

    Ok(())
}

#[test]
fn filter_limit() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .args("--limit 1")
        .arg("003@.0 == '119232022' || 003@.0 == '1004916019'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .run()?;

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .args("--limit 99")
        .arg("003@.0 == '119232022' || 003@.0 == '1004916019'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .args("--limit 0")
        .arg("003@.0 == '119232022' || 003@.0 == '1004916019'")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE1)
        .with_stdout(SAMPLE2)
        .run()?;

    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .args("--limit abc")
        .arg("003@.0 == '119232022' || 003@.0 == '1004916019'")
        .arg("tests/data/dump.dat.gz")
        .with_stderr("error: Invalid limit value, expected unsigned integer.\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn filter_expression_file() -> MatchResult {
    CommandBuilder::new("filter")
        .arg("--skip-invalid")
        .args("--file tests/data/filter.txt")
        .arg("True")
        .arg("tests/data/dump.dat.gz")
        .with_stdout(SAMPLE2)
        .run()?;

    Ok(())
}
