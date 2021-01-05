mod common;

use common::CliRunner;

#[test]
fn cat() {
    let sample1 = include_str!("data/1.dat");
    let sample2 = include_str!("data/2.dat");

    let result = CliRunner::new().invoke("cat", &["tests/data/1.dat"]);
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), sample1);

    let result = CliRunner::new()
        .invoke("cat", &["tests/data/1.dat", "tests/data/2.dat"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", sample1, sample2)
    );

    let result = CliRunner::new().invoke(
        "cat",
        &[
            "--skip-invalid",
            "tests/data/1.dat",
            "tests/data/invalid.dat",
            "tests/data/empty.dat",
            "tests/data/2.dat",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", sample1, sample2)
    );

    let result = CliRunner::new()
        .invoke("cat", &["tests/data/1.dat", "tests/data/invalid.dat"]);
    assert!(!result.status.success());
}
