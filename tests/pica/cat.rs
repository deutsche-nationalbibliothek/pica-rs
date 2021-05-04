use crate::support::{CommandBuilder, MatchResult};

static SAMPLE1: &str = include_str!("../data/1004916019.dat");
static SAMPLE2: &str = include_str!("../data/119232022.dat");

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

    Ok(())
}
