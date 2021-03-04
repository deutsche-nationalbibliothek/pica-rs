mod common;

use common::CliRunner;

#[test]
fn test_select_cmd() {
    let result = CliRunner::new().invoke(
        "select",
        &["--skip-invalid", "003@.0,002@.0", "tests/data/1.dat"],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "123456789X,Tp1\n"
    );

    let result = CliRunner::new().invoke(
        "select",
        &["--skip-invalid", "003@.0,012A/00.a", "tests/data/2.dat"],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        concat!("234567891X,1\n", "234567891X,2\n")
    );

    let result = CliRunner::new().invoke(
        "select",
        &["--skip-invalid", "013B.a,013B/00.c", "tests/data/2.dat"],
    );
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().is_empty());
}

#[test]
fn test_invalid_selector() {
    let result =
        CliRunner::new().invoke("select", &["003!.0", "tests/data/1.dat"]);
    assert!(!result.status.success());
}

#[test]
fn test_skip_invalid() {
    let result = CliRunner::new()
        .invoke("select", &["003@.0", "tests/data/invalid.dat"]);
    assert!(!result.status.success());
}
