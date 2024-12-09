use assert_cmd::Command;

use crate::prelude::*;

#[test]
fn regex() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "002@.0 =~ '^T[bfgpsu][1-3z]$'"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "002@.0 =~ '^T[bfgpsu][23z]$'"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "042A{ ALL a =~ '.*p$' }"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "008A{ ALL a =~ '^[sz]$' }"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-i", "002@.0 =~ '^t[BFGPSU][1-3Z]$'"])
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
fn regex_inverted() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "002@.0 !~ '^O(af|lfo)$'"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-i", "002@.0 !~ '^olfo'"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    Ok(())
}
