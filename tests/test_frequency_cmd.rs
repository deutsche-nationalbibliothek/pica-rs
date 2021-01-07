mod common;

use common::CliRunner;

#[test]
fn test_frequency_cmd() {
    let result = CliRunner::new()
        .invoke("frequency", &["-s", "002@.0", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Tp1,2\nTp2,1\n");

    let result = CliRunner::new().invoke(
        "frequency",
        &["-s", "--limit", "2", "002@.0", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Tp1,2\n");

    let result = CliRunner::new()
        .invoke("frequency", &["002@.0", "tests/data/invalid.dat"]);
    assert!(!result.status.success());
}
