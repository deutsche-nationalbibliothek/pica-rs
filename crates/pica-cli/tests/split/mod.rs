use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn split_default() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["split", "-s", "100"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn split_size() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["split", "-s", "2"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .arg(data_dir().join("ada.dat"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::exists().eval(outdir.child("0.dat").path())
    );
    assert!(
        predicates::path::exists().eval(outdir.child("1.dat").path())
    );
    assert!(
        predicates::path::exists().eval(outdir.child("2.dat").path())
    );
    assert!(
        predicates::path::exists().eval(outdir.child("3.dat").path())
    );
    assert!(
        predicates::path::exists().eval(outdir.child("4.dat").path())
    );

    assert!(predicates::path::eq_file(data_dir().join("ada.dat"))
        .eval(outdir.child("6.dat").path()));

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn split_skip_invalid() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["split", "-s", "5"])
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::eq_file(data_dir().join("ada.dat"))
        .eval(outdir.child("0.dat").path()));
    outdir.close().unwrap();

    let outdir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["split", "10"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn split_gzip() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["split", "-s", "--gzip", "100"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("0.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn split_template() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["split", "-s", "5"])
        .args(["--template", "FOO_{}.dat"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .arg(data_dir().join("ada.dat"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("FOO_0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("5\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("FOO_1.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("5\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("FOO_2.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("3\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}
