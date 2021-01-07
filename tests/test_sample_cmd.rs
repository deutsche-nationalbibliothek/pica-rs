mod common;

use common::CliRunner;

static SAMPLE1: &'static str = include_str!("data/1.dat");
static SAMPLE2: &'static str = include_str!("data/2.dat");
static SAMPLE3: &'static str = include_str!("data/3.dat");

#[test]
fn test_sample_cmd() {
    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "1", "tests/data/all.dat"]);
    assert!(result.status.success());

    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output == SAMPLE1 || output == SAMPLE2 || output == SAMPLE3);

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "2", "tests/data/all.dat"]);
    assert!(result.status.success());

    let output = String::from_utf8(result.stdout).unwrap();
    assert!(
        output == format!("{}{}", SAMPLE1, SAMPLE2)
            || output == format!("{}{}", SAMPLE1, SAMPLE3)
            || output == format!("{}{}", SAMPLE2, SAMPLE1)
            || output == format!("{}{}", SAMPLE2, SAMPLE3)
            || output == format!("{}{}", SAMPLE3, SAMPLE1)
            || output == format!("{}{}", SAMPLE3, SAMPLE2)
    );

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "100", "tests/data/1.dat"]);
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1)
}

#[test]
fn test_invalid_sample_size() {
    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "0", "tests/data/all.dat"]);
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "-1", "tests/data/all.dat"]);
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "a", "tests/data/all.dat"]);
    assert!(!result.status.success());
}

#[test]
fn test_invalid_records() {
    let result =
        CliRunner::new().invoke("sample", &["1", "tests/data/all.dat"]);
    assert!(!result.status.success());
}
