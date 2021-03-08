mod common;

use common::CliRunner;

static SAMPLE1: &'static str = include_str!("data/1.dat");
static SAMPLE2: &'static str = include_str!("data/2.dat");
static SAMPLE3: &'static str = include_str!("data/3.dat");
static SAMPLE4: &'static str = include_str!("data/4.dat");

#[test]
fn test_slice_cmd() {
    let result =
        CliRunner::new().invoke("slice", &["-s", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new()
        .invoke("slice", &["-s", "--start", "1", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE2, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new()
        .invoke("slice", &["-s", "--end", "2", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE2)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &["-s", "--start", "1", "--end", "2", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}", SAMPLE2)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &[
            "-s",
            "--start",
            "1",
            "--length",
            "4",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE2, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &["-s", "--start", "2", "--end", "4", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE3);

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
        format!("{}{}", SAMPLE3, SAMPLE4)
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
