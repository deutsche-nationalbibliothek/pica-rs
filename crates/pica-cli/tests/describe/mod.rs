use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn write_stdout() -> TestResult {
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
fn write_csv() -> TestResult {
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
fn write_tsv() -> TestResult {
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

#[test]
fn skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("describe")
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::starts_with(
            "error: parse error: invalid record on line 1",
        ));

    Ok(())
}

#[test]
fn keep() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "002@"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!("field,0\n002@,12\n", read_to_string(out.path())?);
    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn discard() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "00[23]@", "-d", "003@"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!("field,0\n002@,12\n", read_to_string(out.path())?);
    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn allow() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("ppn\n118540238\n")?;

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "007N"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "field,0,a,v\n007N,15,15,6\n",
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn deny() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let deny = temp_dir.child("DENY.csv");
    deny.write_str("ppn\n118540238\n")?;

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "007N"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "field,0,a,v\n007N,28,28,18\n",
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn r#where() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "007N"])
        .args(["--where", "003@.0 == '118540238'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "field,0,a,v\n007N,15,15,6\n",
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn where_and() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "007N"])
        .args(["--where", "003@.0 == '118540238'"])
        .args(["--and", "002@.0 == 'Tpz'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "field,0,a,v\n007N,15,15,6\n",
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn where_or() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "007N"])
        .args(["--where", "003@.0 == '118540238'"])
        .args(["--or", "002@.0 == 'Tpz'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "field,0,a,v\n007N,15,15,6\n",
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn where_not() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["describe", "-s", "-k", "007N"])
        .args(["--where", "003@.0 == '118540238'"])
        .args(["--not", "002@.0 == 'Tp1'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "field,0,a,v\n007N,15,15,6\n",
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}
