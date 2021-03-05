mod common;

use common::CliRunner;

#[test]
fn test_slice_cmd() {
    let sample1 = include_str!("data/1.dat");
    let sample2 = include_str!("data/2.dat");
    let sample3 = include_str!("data/3.dat");

    let result =
        CliRunner::new().invoke("slice", &["-s", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", sample1, sample2, sample3)
    );

    let result = CliRunner::new()
        .invoke("slice", &["-s", "--start", "1", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", sample2, sample3)
    );

    let result = CliRunner::new()
        .invoke("slice", &["-s", "--end", "2", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", sample1, sample2)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &["-s", "--start", "1", "--end", "2", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}", sample2)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &[
            "-s",
            "--start",
            "1",
            "--length",
            "1",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}", sample2)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &[
            "-s",
            "--start",
            "2",
            "--length",
            "2",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}", sample3)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &[
            "-s",
            "--start",
            "1",
            "--end",
            "2",
            "--length",
            "1",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("slice", &["--start", "2", "tests/data/all.dat.gz"]);
    assert!(!result.status.success());
}
