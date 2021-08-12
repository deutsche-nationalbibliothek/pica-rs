use assert_cmd::Command;
use std::fs::read_to_string;

#[test]
fn pica_cat_single_file() {
    let mut cmd = Command::cargo_bin("pica").unwrap();
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .assert();

    let expected = read_to_string("tests/data/1004916019.dat").unwrap();
    assert.success().stdout(expected);
}

#[test]
fn pica_cat_multiple_file() {
    let mut cmd = Command::cargo_bin("pica").unwrap();
    let assert = cmd
        .arg("cat")
        .arg("--skip-invalid")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/000009229.dat")
        .assert();

    let expected = format!(
        "{}{}",
        read_to_string("tests/data/1004916019.dat").unwrap(),
        read_to_string("tests/data/000009229.dat").unwrap()
    );
    assert.success().stdout(expected);
}
