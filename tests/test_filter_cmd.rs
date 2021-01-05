mod common;

use common::CliRunner;

static SAMPLE1: &'static str = include_str!("data/1.dat");
static SAMPLE2: &'static str = include_str!("data/2.dat");
static SAMPLE3: &'static str = include_str!("data/3.dat");

#[test]
fn test_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 == '123456789X'",
            "tests/data/all.dat",
        ],
    );
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 == '123456789X' || 003@{0 == '234567891X'}",
            "tests/data/all.dat",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE2)
    );
}

#[test]
fn test_invert_match() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "--invert-match",
            "003@.0 == '123456789X'",
            "tests/data/all.dat",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE2, SAMPLE3)
    );
}

#[test]
fn test_invalid_filter() {
    let result = CliRunner::new()
        .invoke("filter", &["003@.! == '0123456789X'", "tests/data/1.dat"]);
    assert!(!result.status.success());
}

#[test]
fn test_skip_invalid() {
    let result = CliRunner::new().invoke(
        "filter",
        &["003@.0 == '0123456789X'", "tests/data/invalid.dat"],
    );
    assert!(!result.status.success());

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 == '123456789X'",
            "tests/data/1.dat",
        ],
    );

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);
}
