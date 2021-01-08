mod common;

use common::CliRunner;
use tempdir::TempDir;

static SAMPLE1: &'static str = include_str!("data/1.dat");
static SAMPLE2: &'static str = include_str!("data/2.dat");
static SAMPLE3: &'static str = include_str!("data/3.dat");

#[test]
fn test_split_cmd_outdir_exists() {
    let tempdir = TempDir::new("split_test").unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "split",
        &[
            "--skip-invalid",
            "--template",
            "SPLIT_{}.dat",
            "--outdir",
            outdir.to_str().unwrap(),
            "2",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());

    let content = std::fs::read_to_string(outdir.join("SPLIT_0.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE1, SAMPLE2));

    let content = std::fs::read_to_string(outdir.join("SPLIT_1.dat")).unwrap();
    assert_eq!(content, SAMPLE3);
}

#[test]
fn test_split_cmd_no_outdir() {
    let tempdir = TempDir::new("split_test").unwrap();
    let outdir = tempdir.path().join("split-test");

    let result = CliRunner::new().invoke(
        "split",
        &[
            "--skip-invalid",
            "--template",
            "SPLIT_{}.dat",
            "--outdir",
            outdir.to_str().unwrap(),
            "2",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());

    let content = std::fs::read_to_string(outdir.join("SPLIT_0.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE1, SAMPLE2));

    let content = std::fs::read_to_string(outdir.join("SPLIT_1.dat")).unwrap();
    assert_eq!(content, SAMPLE3);
}

#[test]
fn test_invalid_chunk_size() {
    let result =
        CliRunner::new().invoke("split", &["0", "tests/data/invalid.dat"]);
    assert!(!result.status.success());

    let result =
        CliRunner::new().invoke("split", &["a", "tests/data/invalid.dat"]);
    assert!(!result.status.success());
}

#[test]
fn test_skip_invalid() {
    let tempdir = TempDir::new("split_test").unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "split",
        &[
            "1",
            "--outdir",
            outdir.to_str().unwrap(),
            "tests/data/invalid.dat",
        ],
    );
    assert!(!result.status.success());
}
