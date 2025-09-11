use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
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

    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(outdir.join("119232022.dat").as_path())
    );

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

    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(outdir.join("28p.dat").as_path())
    );
    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(outdir.join("9.5p.dat").as_path())
    );

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_where() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let outdir = TempDir::new().unwrap();

    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 2);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tp1.dat"))
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_where_and() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let outdir = TempDir::new().unwrap();

    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "003@.0 == '118540238'"])
        .args(["--and", "002@.0 =^ 'Tp'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 1);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_and_or() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let outdir = TempDir::new().unwrap();

    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "003@.0 == '118607626'"])
        .args(["--or", "002@.0 == 'Tpz'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 2);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tp1.dat"))
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_where_not() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let outdir = TempDir::new().unwrap();

    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["--not", "003@.0 == '118607626'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 1);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_allow() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let outdir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("ppn\n118540238\n118607626\n")?;

    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 2);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tp1.dat"))
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_deny() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let outdir = TempDir::new().unwrap();

    let deny = temp_dir.child("DENY.csv");
    deny.write_str(
        "ppn\n\
        040993396\n\
        04099337X\n\
        040991970\n\
        040991989\n\
        041274377\n\
        964262134\n\
        040533093\n\
        040309606\n\
        040128997\n\
        040651053\n",
    )?;

    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 2);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tp1.dat"))
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_filter_set_column() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let outdir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("id\n118540238\n118607626\n")?;

    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-column", "id"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 2);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tp1.dat"))
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_filter_set_source() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let outdir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("bbg\nTp1\nTpz\n")?;

    let assert = cmd
        .args(["partition", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-source", "002@.0"])
        .args(["--filter-set-column", "bbg"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 2);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tp1.dat"))
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    outdir.close().unwrap();
    Ok(())
}

#[test]
fn partition_limit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let outdir = TempDir::new().unwrap();

    let assert = cmd
        .args(["partition", "-s", "-l", "2", "002@.0"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(outdir.read_dir().unwrap().count(), 2);

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("Tp1.dat"))
        .arg(outdir.join("Tpz.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}
