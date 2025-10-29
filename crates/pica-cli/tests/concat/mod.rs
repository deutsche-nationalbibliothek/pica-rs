use std::fs::read_to_string;

use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn single_file_write_stdout() -> TestResult {
    let mut cmd = pica_cmd();
    let assert =
        cmd.arg("concat").arg(data_dir().join("ada.dat")).assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn read_stdin() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("concat")
        .write_stdin(read_to_string(data_dir().join("ada.dat"))?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("cat")
        .arg("-")
        .write_stdin(read_to_string(data_dir().join("ada.dat"))?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["concat", "-s", "-u"])
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg("-")
        .write_stdin(read_to_string(data_dir().join("ada.dat"))?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn single_file_write_file() -> TestResult {
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .arg("concat")
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(out.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn write_gzip() -> TestResult {
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat.gz");

    let assert = cmd
        .arg("concat")
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd.arg("concat").arg(out.as_os_str()).assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn single_file_write_tee() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");
    let tee = temp_dir.child("tee.dat");

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("concat")
        .args(["--tee", tee.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(tee.path())
    );

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("concat")
        .args(["--tee", tee.to_str().unwrap()])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(tee.path())
    );
    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(out.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn skip_invalid() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("concat")
        .arg("--skip-invalid")
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["concat", "-s"])
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("concat")
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    // config
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let config = temp_dir.child("pica.toml");
    let filename = config.to_str().unwrap();

    let assert = cmd
        .args(["--config", filename])
        .arg("config")
        .args(["skip-invalid", "true"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["-c", config.to_str().unwrap()])
        .arg("concat")
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn unique_by_idn() -> TestResult {
    let mut cmd = pica_cmd();

    let assert = cmd
        .args(["concat", "-u", "--unique-strategy", "idn"])
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn unique_by_hash() -> TestResult {
    let mut cmd = pica_cmd();

    let assert = cmd
        .args(["concat", "--unique"])
        .args(["--unique-strategy", "hash"])
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn append() -> TestResult {
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .args(["concat", "--append", "-o", out.to_str().unwrap()])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("ada.dat"))?.trim_end(),
        read_to_string(out.path())?.lines().nth(1).unwrap()
    );

    Ok(())
}
