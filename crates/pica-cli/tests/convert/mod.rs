use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

use super::prelude::*;

#[test]
fn convert_from_plus_to_xml() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("ada.xml");

    let assert = cmd
        .arg("convert")
        .args(["--from", "plus", "--to", "xml"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("ada.xml"))?,
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn convert_from_plus_to_json() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("ada.json");

    let assert = cmd
        .arg("convert")
        .args(["--from", "plus", "--to", "json"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("ada.json"))?,
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn convert_from_plus_to_plus() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("ada.dat");

    let assert = cmd
        .arg("convert")
        .args(["--from", "plus", "--to", "plus"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("ada.dat"))?,
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn convert_from_plus_to_plain() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("ada.plain");

    let assert = cmd
        .arg("convert")
        .args(["--from", "plus", "--to", "plain"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("ada.plain"))?,
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn convert_from_plus_to_import() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("ada.import");

    let assert = cmd
        .arg("convert")
        .args(["--from", "plus", "--to", "import"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("ada.import"))?,
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn convert_from_plus_to_binary() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("ada.bin");

    let assert = cmd
        .arg("convert")
        .args(["--from", "plus", "--to", "binary"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("ada.bin"))?,
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn convert_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["convert", "-s"])
        .args(["--from", "plus", "--to", "json"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("[]"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["convert"])
        .args(["--from", "plus", "--to", "json"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty().not())
        .stderr(predicates::str::contains(
            "parse erorr: invalid record on line 1",
        ));

    Ok(())
}
