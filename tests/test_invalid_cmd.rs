mod common;

use common::CliRunner;

static INVALID: &'static str = include_str!("data/invalid.dat");

#[test]
fn test_invalid_cmd() {
    let result =
        CliRunner::new().invoke("invalid", &["tests/data/invalid.dat"]);
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), INVALID);

    let result = CliRunner::new().invoke("invalid", &["tests/data/all.dat"]);
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), INVALID);

    let result = CliRunner::new().invoke("invalid", &["tests/data/1.dat"]);
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().is_empty());
}
