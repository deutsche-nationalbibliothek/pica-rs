use std::fs::read_to_string;

use assert_cmd::Command;
// use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn partition_by_bbg() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["partition", "-s", "002@.0"])
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
        .arg(outdir.join("Tg1.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tu1.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("6\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn partition_gzip() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["partition", "-s", "--gzip", "002@.0"])
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
        .arg(outdir.join("Tp1.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tu1.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("6\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn partition_template() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .args(["--template", "BBG_{}.dat"])
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
        .arg(outdir.join("BBG_Tg1.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("BBG_Tu1.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("6\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn partition_stdin() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["partition", "003@.0"])
        .args(["-o", outdir.to_str().unwrap()])
        .write_stdin(read_to_string(data_dir().join("ada.dat"))?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::eq_file(data_dir().join("ada.dat"))
        .eval(outdir.join("119232022.dat").as_path()));

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_skip_invalid() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["partition", "003@.0"])
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
fn multiple_partitions() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["partition", "042A.a"])
        .arg(data_dir().join("ada.dat"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::eq_file(data_dir().join("ada.dat"))
        .eval(outdir.join("28p.dat").as_path()));
    assert!(predicates::path::eq_file(data_dir().join("ada.dat"))
        .eval(outdir.join("9.5p.dat").as_path()));

    outdir.close().unwrap();
    Ok(())
}
