use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn allow_list_ppn() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("ppn\n118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn allow_list_idn() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("idn\n118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn allow_list_column() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("xyz\n118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["--filter-set-column", "xyz"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn allow_list_source() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("wd_id\nQ5879").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["--filter-set-column", "wd_id"])
        .args(["--filter-set-source", "006Y{ 0 | S == 'wikidata' }"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn allow_list_mulval() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("DENY.csv");
    allow.write_str("code\nx\nf").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-column", "code"])
        .args(["--filter-set-source", "008A.a"])
        .arg(data_dir().join("goethe.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("DENY.csv");
    allow.write_str("code\nx\ny").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-column", "code"])
        .args(["--filter-set-source", "008A.a"])
        .arg(data_dir().join("goethe.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn allow_list_empty() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("ppn").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn deny_list_ppn() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("DENY.csv");
    deny.write_str("ppn\n118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("goethe.dat"))
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
fn deny_list_idn() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("DENY.csv");
    deny.write_str("idn\n118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("goethe.dat"))
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
fn deny_list_column() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("DENY.csv");
    deny.write_str("xyz\n118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .args(["--filter-set-column", "xyz"])
        .arg(data_dir().join("goethe.dat"))
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
fn deny_list_source() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("DENY.csv");
    deny.write_str("isni\n0000 0001 2099 9104").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .args(["--filter-set-column", "isni"])
        .args(["--filter-set-source", "006Y{ 0 | S == 'isni' }"])
        .arg(data_dir().join("goethe.dat"))
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
fn deny_list_mulval() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("DENY.csv");
    deny.write_str("code\nx\nf").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .args(["--filter-set-column", "code"])
        .args(["--filter-set-source", "008A.a"])
        .arg(data_dir().join("goethe.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("DENY.csv");
    deny.write_str("code\nx\ny").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .args(["--filter-set-column", "code"])
        .args(["--filter-set-source", "008A.a"])
        .arg(data_dir().join("goethe.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn deny_list_empty() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let deny = temp_dir.child("DENY.csv");
    deny.write_str("ppn").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}
