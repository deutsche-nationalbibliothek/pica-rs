use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn split_default() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = pica_cmd();
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

    let mut cmd = pica_cmd();
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

    let mut cmd = pica_cmd();
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

    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(outdir.child("6.dat").path())
    );

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn split_skip_invalid() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = pica_cmd();
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

    assert!(
        predicates::path::eq_file(data_dir().join("ada.dat"))
            .eval(outdir.child("0.dat").path())
    );
    outdir.close().unwrap();

    let outdir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();
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

    let mut cmd = pica_cmd();
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

    let mut cmd = pica_cmd();
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

    let mut cmd = pica_cmd();
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

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("FOO_0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("5\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(outdir.join("FOO_1.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("5\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
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

#[test]
fn split_where() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["split", "-s", "100"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "002@.0 == 'Ts1'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(outdir.join("0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040309606\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn split_where_and() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["split", "-s", "100"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "002@.0 == 'Ts1'"])
        .args(["--and", "004B.a == 'saz'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(outdir.join("0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040309606\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn split_where_not() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["split", "-s", "100"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "002@.0 =^ 'Ts'"])
        .args(["--not", "002@.0 =$ 'z'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(outdir.join("0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040309606\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn split_where_and_not() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["split", "-s", "100"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "002@.0 =^ 'Ts'"])
        .args(["--and", "004B.a == 'saz'"])
        .args(["--not", "002@.0 =$ 'z'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(outdir.join("0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040309606\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn split_where_or() -> TestResult {
    let outdir = TempDir::new().unwrap();

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["split", "-s", "1"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "002@.0 == 'Ts1'"])
        .args(["--or", "002@.0 == 'Tg1'"])
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(outdir.join("0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040309606\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(outdir.join("1.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040651053\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn split_allow() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let outdir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("idn\n118540238\n118515551\n")?;

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["split", "-s", "10"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(outdir.join("0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("118540238\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    outdir.close().unwrap();

    Ok(())
}

#[test]
fn split_deny() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let outdir = TempDir::new().unwrap();

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

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["split", "-s", "10"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", outdir.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(outdir.join("0.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("118540238\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    outdir.close().unwrap();

    Ok(())
}
