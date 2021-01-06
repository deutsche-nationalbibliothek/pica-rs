mod common;

use common::CliRunner;
use tempdir::TempDir;

static SAMPLE1: &'static str = include_str!("data/1.dat");
static SAMPLE2: &'static str = include_str!("data/2.dat");
static SAMPLE3: &'static str = include_str!("data/3.dat");

#[test]
fn test_partition_cmd_outdir_exists() {
    let tempdir = TempDir::new("partition_test").unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "partition",
        &[
            "--skip-invalid",
            "--outdir",
            outdir.to_str().unwrap(),
            "002@.0",
            "tests/data/all.dat",
        ],
    );

    assert!(result.status.success());

    let content = std::fs::read_to_string(outdir.join("Tp1.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE1, SAMPLE3));

    let content = std::fs::read_to_string(outdir.join("Tp2.dat")).unwrap();
    assert_eq!(content, SAMPLE2);
}

#[test]
fn test_partition_cmd_no_outdir() {
    let tempdir = TempDir::new("partition_test").unwrap();
    let outdir = tempdir.path().join("part-test");

    let result = CliRunner::new().invoke(
        "partition",
        &[
            "--skip-invalid",
            "--outdir",
            outdir.to_str().unwrap(),
            "002@.0",
            "tests/data/all.dat",
        ],
    );

    assert!(result.status.success());

    let content = std::fs::read_to_string(outdir.join("Tp1.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE1, SAMPLE3));

    let content = std::fs::read_to_string(outdir.join("Tp2.dat")).unwrap();
    assert_eq!(content, SAMPLE2);
}

#[test]
fn test_skip_invalid() {
    let result = CliRunner::new()
        .invoke("partition", &["002@.0", "tests/data/invalid.dat"]);

    assert!(!result.status.success());
}
