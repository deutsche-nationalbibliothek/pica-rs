mod common;

use common::CliRunner;

#[test]
fn test_print_cmd() {
    let result = CliRunner::new().invoke("print", &["tests/data/1.dat"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "003@ $0 123456789X\n002@ $0 Tp1\n012A/00 $a 1 $a 2 $b 1\n\n"
    );

    let result = CliRunner::new().invoke("print", &["tests/data/invalid.dat"]);
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("print", &["--skip-invalid", "tests/data/invalid.dat"]);

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "");

    let result = CliRunner::new()
        .invoke("print", &["--skip-invalid", "tests/data/empty.dat"]);

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "");
}
