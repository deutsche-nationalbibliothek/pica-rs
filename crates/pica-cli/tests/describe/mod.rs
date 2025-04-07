use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn describe_write_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "050C"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::contains("│ 050C  ┆ 7 ┆ 35 │"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn describe_write_csv() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "050C"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!("field,5,a\n050C,7,35\n", read_to_string(out.path())?);
    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn describe_write_tsv() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.tsv");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "050C"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "field\t5\ta\n050C\t7\t35\n",
        read_to_string(out.path())?
    );
    temp_dir.close().unwrap();
    Ok(())
}
