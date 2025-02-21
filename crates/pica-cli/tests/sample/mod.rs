use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn sample_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "1"])
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
fn sample_output() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "1"])
        .arg(data_dir().join("ada.dat"))
        .args(["--output", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_gzip() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat.gz");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "2"])
        .arg(data_dir().join("ada.dat"))
        .args(["--output", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(samples.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "10"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty().not())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "10"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    Ok(())
}

#[test]
fn sample_seed() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "2"])
        .args(["--seed", "1234"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("samples.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_where() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "23"])
        .args(["--where", "003@.0 == '118540238'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("goethe.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_where_and() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "23"])
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["--and", "003@.0 == '118540238'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("goethe.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_where_not() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "23"])
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["--not", "003@.0 == '118607626'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("goethe.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_where_and_not() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "23"])
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["--and", "003@.0 == '118540238'"])
        .args(["--not", "002@.0 == 'Tp3'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("goethe.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_where_or() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "23"])
        .args(["--where", "003@.0 == '118515551'"])
        .args(["--or", "003@.0 == '118540238'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("goethe.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_allow() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("idn\n118540238\n118515551\n")?;

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "10"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("goethe.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn sample_deny() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let samples = temp_dir.child("samples.dat");

    let deny = temp_dir.child("DENY.csv");
    deny.write_str(
        "idn\n\
        118607626\n\
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
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["sample", "-s", "10"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", samples.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(
        predicates::path::eq_file(data_dir().join("goethe.dat"))
            .eval(samples.path())
    );

    temp_dir.close().unwrap();
    Ok(())
}
