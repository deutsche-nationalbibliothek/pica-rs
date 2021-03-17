mod common;

use common::CliRunner;

static SAMPLE1: &'static str = include_str!("data/1.dat");
static SAMPLE2: &'static str = include_str!("data/2.dat");
static SAMPLE3: &'static str = include_str!("data/3.dat");
static SAMPLE4: &'static str = include_str!("data/4.dat");
static SAMPLE5: &'static str = include_str!("data/5.dat");

#[test]
fn test_eq_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 == '123456789X'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);
}

#[test]
fn test_strict_eq_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "012A/*.a == '1'", "tests/data/5.dat"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE5);
}

#[test]
fn test_ne_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 != '123456789X'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE2, SAMPLE3, SAMPLE4)
    );
}

#[test]
fn test_starts_with_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "003@.0 =^ '123'", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);
}

#[test]
fn test_ends_with_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "002@.0 =$ 'p2'", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE2);
}

#[test]
fn test_regex_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@.0 =~ '^Tp[12]$'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );
}

#[test]
fn test_in_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@.0 in ['Tp1', 'Tp3']",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 in ['Tp1', 'Tp3']}",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE3)
    );
}

#[test]
fn test_or_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 == 'T\n1\\ ' || 0 == 'Tp1' || 0 == 'Tp3'}",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 == 'Tp1' || 0 == 'Tp3'} || 003@.0 == '234567891\u{0058}'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );
}

#[test]
fn test_and_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 =^ 'Tp' && 0 =$ '2'}",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE2);

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 =^ 'Tp' && 0 =$ '2'} && 003@.0 == '234567891X'",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE2);
}

#[test]
fn test_grouped_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{ (0 =^ 'Tp' && 0 =$ '2') || 0 == 'Tp1' }",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 =$ 'X' && (002@{0 =^ 'Tp' && 0 =$ '2'} || 002@.0 == 'Tp1')",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );
}

#[test]
fn test_not_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{!(0 == 'Tp2' || c?)}",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "!(002@.0 == 'Tp2' || 002@.0 == 'Tp3')",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE3, SAMPLE4)
    );
}

#[test]
fn test_exists_filter() {
    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "002@.0?", "tests/data/all.dat.gz"],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0? && 0 == 'Tp2'}",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE2);

    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "012A/00?", "tests/data/all.dat.gz"],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "013B?", "tests/data/all.dat.gz"],
    );

    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().is_empty(),);
}

#[test]
fn test_invert_match() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "--invert-match",
            "003@.0 == '123456789X'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE2, SAMPLE3, SAMPLE4)
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
