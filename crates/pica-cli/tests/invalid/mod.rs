use std::fs::read_to_string;

use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn read_file_write_file() -> TestResult {
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .arg("invalid")
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("invalid.dat"))
            .eval(out.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn read_stdin_write_file() -> TestResult {
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .arg("invalid")
        .args(["-o", out.to_str().unwrap()])
        .write_stdin(read_to_string(data_dir().join("invalid.dat"))?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("invalid.dat"))
            .eval(out.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn read_file_write_stdout() -> TestResult {
    let mut cmd = pica_cmd();
    let input = data_dir().join("invalid.dat");

    let assert = cmd.arg("invalid").arg(&input).assert();
    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(&input))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn read_stdin_write_stdout() -> TestResult {
    let mut cmd = pica_cmd();
    let filename = data_dir().join("invalid.dat");

    let assert = cmd
        .arg("invalid")
        .write_stdin(read_to_string(&filename)?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(filename))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn read_multiple_files() -> TestResult {
    let mut cmd = pica_cmd();
    let input = data_dir().join("invalid.dat");

    let assert = cmd
        .arg("invalid")
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();
    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(&input))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn read_gzip_file() -> TestResult {
    let mut cmd = pica_cmd();
    let input = data_dir().join("invalid.dat");

    let assert = cmd
        .arg("invalid")
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();
    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(&input))
        .stderr(predicates::str::is_empty());

    Ok(())
}
